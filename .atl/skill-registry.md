# Skill Registry

## Available Skills

| Name            | Trigger                                                                                                          | Purpose                                                                             |
| --------------- | ---------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| `skill-creator` | When user asks to create a new skill, add agent instructions, or document patterns for AI                        | Creates new AI agent skills following the Agent Skills spec                         |
| `go-testing`    | When writing Go tests, using teatest, or adding test coverage                                                    | Go testing patterns for Gentleman.Dots, including Bubbletea TUI testing             |
| `sdd-explore`   | When the orchestrator launches you to think through a feature, investigate the codebase, or clarify requirements | Explore and investigate ideas before committing to a change                         |
| `sdd-propose`   | When the orchestrator launches you to create or update a proposal for a change                                   | Create a change proposal with intent, scope, and approach                           |
| `sdd-spec`      | When the orchestrator launches you to write or update specs for a change                                         | Write specifications with requirements and scenarios (delta specs for changes)      |
| `sdd-design`    | When the orchestrator launches you to write or update the technical design for a change                          | Create technical design document with architecture decisions and approach           |
| `sdd-tasks`     | When the orchestrator launches you to create or update the task breakdown for a change                           | Break down a change into an implementation task checklist                           |
| `sdd-apply`     | When the orchestrator launches you to implement one or more tasks from a change                                  | Implement tasks from the change, writing actual code following the specs and design |
| `sdd-verify`    | When the orchestrator launches you to verify a completed (or partially completed) change                         | Validate that implementation matches specs, design, and tasks                       |
| `sdd-archive`   | When the orchestrator launches you to archive a change after implementation and verification                     | Sync delta specs to main specs and archive a completed change                       |
| `sdd-init`      | When user wants to initialize SDD in a project, or says "sdd init", "iniciar sdd", "openspec init"               | Initialize Spec-Driven Development context in any project                           |

## Project Conventions

| File                   | Purpose                                                                                                                                                                    |
| ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `AGENTS.md`            | Project-specific AI instructions for the `smt` project                                                                                                                     |
| `.ai/AGENTS_GLOBAL.md` | Global AI guidelines and best practices                                                                                                                                    |
| `.ai/PLAN.md`          | Complete project plan with requirements, CLI interface, sorting behavior, error handling, testing strategy, architecture overview, crate dependencies                      |
| `.ai/ARCHITECTURE.md`  | Detailed architecture document with module dependency diagram, data flow, Rust data structures, parser state machine, atomic write strategy, and comment parsing algorithm |
| `openspec/config.yaml` | SDD project configuration with context and development rules                                                                                                               |
