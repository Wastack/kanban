# kanban-cli
A simple yet elegant kanban board to get things done.

Inspired by `taskwarrior`. If you prefer a kanban board instead of a simple
todo list, then this is your tool.

```bash
> ka
Analysis
3: Add labels to kanban-cli
4: Support Jira integration with kanban-cli

Open
7: Move issues up or down with kanban-cli
8: Buy a new backpack

In Progress
10: Write a REAMDE.md for kanban-cli

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
> ka prio up 3 # Move the issue in it's category. up/down/top/bottom
> ka delete 3
> ka show done # Show all done issues (`ka` only shows the 4 most recently closed issues)
```

### Other highlights

- Issues in Open that lasts more then 2 weeks are marked as overdue, and highlighted with red color (do not sit on your tasks).
- Issues are stored in `$HOME/.kanban`, so that you can transfer your kanban to another machine.