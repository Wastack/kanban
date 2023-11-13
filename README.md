# kanban-cli
A simple yet elegant kanban board to get things done.

Inspired by `taskwarrior`. If you prefer a kanban board instead of a simple
todo list, then this is your tool.

```bash
> ka
Open
7: Move issues up or down with kanban-cli
8: Buy a new backpack

Review
2: Implement kanban-cli edit functionality

Done
0: Implement kanban-cli add
1: Design kanban-cli subcommands
5: Implement kanban-cli edit
6: Support moving issues for kanban-cli
...
```

```bash
> ka add "New open issue"
> ka add "New issue in analysis" analysis
> ka move done 3
> ka move done 3 4 5
> ka edit 3  # Interactive editing of the issue with $EDITOR (defaults to vim)
> ka prio up 3 # Move the issue up 1 in its category. up/down/top/bottom
> ka delete 3 2
```

### Other highlights

- Issues in Open that lasts more then 2 weeks are marked as overdue, and highlighted with red color (do not sit on your tasks).
- Issues are stored in `$HOME/.kanban`. This allows you to transfer your kanban to another machine.