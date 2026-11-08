import type {
  AiStatus,
  CoreClient,
  SubmitTaskRequest,
  Task,
  TaskEventRecord,
  TaskRevertReport,
} from '@se/protocol';

/** What the AI feature needs from the core client. */
export type AiClient = Pick<CoreClient, 'send'>;

/** What is known about the AI provider (checked by the core the first time). */
export async function getAiStatus(client: AiClient): Promise<AiStatus> {
  return (await client.send({ kind: 'getAiStatus' })).status;
}

/** Checks the AI provider again (after installing it or logging in). */
export async function refreshAiStatus(client: AiClient): Promise<AiStatus> {
  return (await client.send({ kind: 'refreshAiStatus' })).status;
}

/** Submits a task; resolves with it, queued. */
export async function submitTask(client: AiClient, request: SubmitTaskRequest): Promise<Task> {
  return (await client.send({ kind: 'submitTask', ...request })).task;
}

/** Cancels a queued or running task. */
export async function cancelTask(client: AiClient, projectId: string, taskId: string) {
  await client.send({ kind: 'cancelTask', projectId, taskId });
}

/** Runs a failed or cancelled task's last prompt again. */
export async function retryTask(client: AiClient, projectId: string, taskId: string) {
  return (await client.send({ kind: 'retryTask', projectId, taskId })).task;
}

/** Sends a new prompt to a finished task's agent. */
export async function followUpTask(
  client: AiClient,
  projectId: string,
  taskId: string,
  prompt: string,
): Promise<Task> {
  return (await client.send({ kind: 'followUpTask', projectId, taskId, prompt })).task;
}

/** Undoes a finished task's edits that nobody changed since; resolves with what was left. */
export async function revertTask(
  client: AiClient,
  projectId: string,
  taskId: string,
): Promise<TaskRevertReport> {
  return (await client.send({ kind: 'revertTask', projectId, taskId })).report;
}

/** The tasks of an open project, oldest first. */
export async function listTasks(client: AiClient, projectId: string): Promise<Task[]> {
  return (await client.send({ kind: 'listTasks', projectId })).tasks;
}

/** The event log of a task. */
export async function getTaskEvents(
  client: AiClient,
  projectId: string,
  taskId: string,
): Promise<TaskEventRecord[]> {
  return (await client.send({ kind: 'getTaskEvents', projectId, taskId })).events;
}
