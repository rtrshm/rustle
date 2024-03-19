#### rustle
WIP Terminal-based journaling tool
###### Controls
- `q`: quit
- `c`: open calendar
- `TAB`: switch between menu and editor
- `ctrl` + `n`: create new file
- `ctrl` + `s`: save current file (with editbox in focus)
###### Info
- Entries are stored under /entries/`[date]`

###### Dependencies
- crossterm: Rust terminal backend
- ratatui: Console GUI framework
- tui_textarea: Editbox GUI element for ratatui 
- log; simple_logging: logging frameworks (write to test.log) by default

###### Known Issues
- New filenames get a random space added sometimes
- Saving requires editbox in focus