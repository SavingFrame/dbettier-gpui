# Connection editor UI

`connection_editor::open(profile, cx)` returns a window handle and a `ConnectionEditor` entity. Pass `None` for Add or an existing profile for Edit. The window owns its editor and does not depend on a workspace entity.

## Module layout

- `mod.rs`: module declarations, public interface, actions, and keybinding initialization.
- `editor.rs`: editor entity, input state, and request handling.
- `model.rs`: drafts, input modes, statuses, and events.
- `view.rs`: form and footer rendering.
- `window.rs`: window creation and native close handling.

## Logic integration

Subscribe to `ConnectionEditorEvent` on the returned entity and retain the GPUI subscription:

- `SaveRequested(ConnectionDraft)`: validate and persist with your implementation.
- `TestRequested(ConnectionDraft)`: run your connection test.
- `Cancelled`: Cancel, Escape, or the native close button. Handle outstanding operations as appropriate.

Update the entity with `set_status(status, cx)` to show `Testing`, `Saving`, `Success(message)`, or `Error(message)`. Busy states disable fields, mode switching, Save, and Test. Cancel remains available. Close the returned window after a successful save using its handle, handling any update error if the window has already closed.

The default request handlers display an honest not-configured notice, then emit the event. No database, filesystem, persistence, URI parsing, validation, or async-operation logic is implemented here. Subscribers can replace the notice with their own status.

## Form behavior

- Drafts contain raw strings, including the port. Validation and normalization belong to the consumer.
- Existing profiles open in URI mode and preserve the original URI exactly.
- Fields and URI modes retain independent values. Only the selected mode is included in a request; there is no automatic conversion.
- Fields mode includes an SSL selector. URI mode uses SSL settings supplied in the URI.
- Password and URI inputs are masked with reveal buttons. Drafts contain credentials and must not be logged.
- Escape cancels. Ctrl+Enter or Cmd+Enter requests Save. Tab navigation uses the standard GPUI controls.
