import { Writable } from 'node:stream';

/** Write one completed archive, await its callback and drain, and keep the stream open. */
export function writeBuffer(stream: Writable, bytes: Buffer): Promise<void> {
  return new Promise((resolve, reject) => {
    let completed = false,
      drained = false,
      settled = false;
    const cleanup = () => {
      stream.off('error', fail);
      stream.off('close', closed);
      stream.off('drain', drain);
    };
    const fail = (error: Error) => {
      if (!settled) {
        settled = true;
        cleanup();
        reject(error);
      }
    };
    const closed = () => fail(new Error('Writable closed before the archive was written'));
    const finish = () => {
      if (completed && drained && !settled) {
        settled = true;
        cleanup();
        resolve();
      }
    };
    const drain = () => {
      drained = true;
      finish();
    };
    stream.once('error', fail);
    stream.once('close', closed);
    stream.once('drain', drain);
    try {
      drained = stream.write(bytes, (error) => {
        // Node emits 'error' after invoking this callback. Leave the listener in
        // place until then so a write failure cannot become an uncaught event.
        if (error) {
          queueMicrotask(() => fail(error));
          return;
        }
        completed = true;
        finish();
      });
      finish();
    } catch (error) {
      fail(error instanceof Error ? error : new Error(String(error)));
    }
  });
}
