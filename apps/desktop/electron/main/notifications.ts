import { BrowserWindow, Notification, type WebContents } from 'electron';

import { IPC_CHANNELS, type SystemNotification } from '../shared/bridge-api';

/** Notifications shown and not yet clicked or closed: kept so they are not garbage collected. */
const shown = new Set<Notification>();

/**
 * Shows a system notification for the renderer of `sender`; a click brings its window to the
 * front and sends the notification's tag back (`IPC_CHANNELS.notificationClicked`).
 */
export function showSystemNotification(sender: WebContents, notification: SystemNotification) {
  if (!Notification.isSupported()) return;
  const native = new Notification({ title: notification.title, body: notification.body });
  const forget = () => shown.delete(native);
  native.on('click', () => {
    forget();
    if (sender.isDestroyed()) return;
    const window = BrowserWindow.fromWebContents(sender);
    if (window !== null) {
      if (window.isMinimized()) window.restore();
      window.show();
      window.focus();
    }
    sender.send(IPC_CHANNELS.notificationClicked, notification.tag);
  });
  native.on('close', forget);
  shown.add(native);
  native.show();
}
