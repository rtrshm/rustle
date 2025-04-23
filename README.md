## rustle
WIP Terminal-based journaling tool
###### Controls
- `q`: quit
- `c`: open calendar
- `TAB`: switch between menu and editor
- `ctrl` + `n`: create new file
- `ctrl` + `s`: save current file (with editbox in focus)
###### Info
- Entries are stored under ./entries/`[date]`

###### Dependencies
- crossterm: Rust terminal backend
- ratatui: Console GUI framework
- tui_textarea: Editbox GUI element for ratatui 
- log; simple_logging: logging frameworks (write to test.log by default)

###### TODO
- filename creation textboxes must forget their content
- keep entries that have been interacted with in a session in memory so progress is not lost when switching entries without saving
