use crate::render::Renderer;
use crate::state::AppState;
use super::events::{self, Event};
use git_rewind_cli::app::service::AppService;
use git_rewind_git::repository::RepositoryHandle;
use ratatui::Terminal;
use ratatui::backend::Backend;
use std::io;
use std::time::Duration;

/// Runs the application loop with a custom event supplier.
/// Decouples backend service orchestration from layout rendering.
pub fn run_with_events<B: Backend, F>(
    terminal: &mut Terminal<B>,
    state: &mut AppState,
    service: &AppService,
    repo: &RepositoryHandle,
    mut next_event: F,
) -> io::Result<()>
where
    F: FnMut() -> io::Result<Option<Event>>,
{
    loop {
        // 1. Synchronously load details and diff for the selected commit if needed
        if let Some(selected_item) = state.timeline.selected_item() {
            let selected_commit_id = selected_item.commit.clone();
            let need_load = match &state.timeline.selected_commit_details {
                Some(details) => details.id != selected_commit_id,
                None => true,
            };

            if need_load {
                let details_res = service.inspect_commit(repo, selected_commit_id.clone());
                let diff_res = service.inspect_diff(repo, selected_commit_id);

                if let (Ok(details), Ok(diff)) = (details_res, diff_res) {
                    let action = crate::actions::Action::UpdateCommitDetails { details, diff };
                    crate::actions::reduce(state, action);
                }
            }
        }

        // 2. Draw current layout frame
        terminal.draw(|f| {
            Renderer::render(f, state);
        })?;

        // 3. Retrieve and dispatch input events
        if let Some(action) = next_event()?.and_then(|event| crate::actions::map_event_to_action(event, state)) {
            if let crate::actions::Action::TriggerReset = action {
                let is_dirty = service.is_dirty(repo).unwrap_or(false);
                let action = crate::actions::Action::ShowConfirmReset { is_dirty };
                crate::actions::reduce(state, action);
            } else {
                let result = crate::actions::reduce(state, action);
                match result {
                    crate::actions::ReduceResult::Quit => break,
                    crate::actions::ReduceResult::ResetRepository { commit_id, hard } => {
                        let reset_res = service.reset_repository(repo, &commit_id, hard);
                        match reset_res {
                            Ok(_) => {
                                // Refresh reflog timeline after travelling state
                                if let Ok(items) = service.load_timeline(repo) {
                                    let action = crate::actions::Action::SetTimelineItems(items);
                                    crate::actions::reduce(state, action);
                                }
                            }
                            Err(err) => {
                                let action = crate::actions::Action::ShowError {
                                    title: "Rewind Failed".to_string(),
                                    message: err.to_string(),
                                };
                                crate::actions::reduce(state, action);
                            }
                        }
                    }
                    crate::actions::ReduceResult::Continue => {}
                }
            }
        }
    }
    Ok(())
}

/// Starts the application rendering and event polling loop.
/// Calls draw iteratively and polls for terminal inputs, exiting on Event::Quit.
pub fn run<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &mut AppState,
    service: &AppService,
    repo: &RepositoryHandle,
) -> io::Result<()> {
    run_with_events(terminal, state, service, repo, || {
        events::poll_event(Duration::from_millis(100))
    })
}
