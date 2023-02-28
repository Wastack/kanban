# kanban-cli
A simple yet elegant kanban board to get things done in terminal.

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
9: Define persistence model for kanban-cli
```

```bash
> ka add "New open issue"
> ka add "New issue in analysis" analysis
> ka move 3 done
> ka edit 3  # Interactive editing of the issue with $EDITOR (defaults to vim)
> ka delete 3
```
