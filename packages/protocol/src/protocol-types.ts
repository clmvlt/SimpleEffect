/**
 * Types derived from the generated protocol types, without any hand-written table: every
 * `Command` has a `CommandResult` variant with the same `kind` (ADR-0012).
 */
import type { Command, CommandResult, CoreEvent } from './generated';

/** `kind` of every command, e.g. `'ping' | 'getCoreInfo'`. */
export type CommandKind = Command['kind'];

/** The command whose `kind` is `K`. */
export type CommandOf<K extends CommandKind> = Extract<Command, { kind: K }>;

/** The result answering the command whose `kind` is `K`. */
export type ResultFor<K extends CommandKind> = Extract<CommandResult, { kind: K }>;

/** `kind` of every event pushed by the core. */
export type CoreEventKind = CoreEvent['kind'];

/** The event whose `kind` is `K`. */
export type CoreEventOf<K extends CoreEventKind> = Extract<CoreEvent, { kind: K }>;

/** Command kinds without a result variant of the same kind: must stay `never` (checked in tests). */
export type CommandsWithoutResult = Exclude<CommandKind, CommandResult['kind']>;
