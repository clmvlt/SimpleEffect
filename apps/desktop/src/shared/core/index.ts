/**
 * The renderer's connection to se-core: one typed `CoreClient` (from `@se/protocol`) created at
 * startup and shared through React context.
 */
export { createCoreClient } from './create-core-client';
export { CoreClientProvider } from './CoreClientProvider';
export type { CoreClientProviderProps } from './CoreClientProvider';
export { useConnectionStatus } from './use-connection-status';
export { useCoreClient } from './use-core-client';
export { useCoreEndpoint } from './use-core-endpoint';
export { useCoreEvent } from './use-core-event';
export { useCoreInfo } from './use-core-info';
