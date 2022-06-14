use std::time::Duration;

use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::error::Error;
use tokio_stream::StreamExt;
use tokio_util::time::delay_queue::{DelayQueue, Expired};

pub type DSender<T> = Sender<(T, Duration)>;
pub type DReceiver<T> = Receiver<Result<Expired<T>, Error>>;

pub trait DelayQueueChan<T> {
    fn channel(self) -> (DSender<T>, DReceiver<T>)
    where
        T: Send + 'static;
}

impl<T> DelayQueueChan<T> for DelayQueue<T> {
    fn channel(self) -> (DSender<T>, DReceiver<T>)
    where
        T: Send + 'static,
    {
        let (in_tx, mut in_rx) = channel::<(T, Duration)>(64);
        let (out_tx, out_rx) = channel::<Result<Expired<T>, Error>>(64);
        let mut dq = self;

        tokio::spawn(async move {
            let mut empty = true;
            let mut in_closed = false;
            let mut out_closed = false;

            loop {
                if in_closed && empty {
                    // 输入关掉了，且dq为空，退出
                    break;
                }

                if out_closed {
                    // 输出关掉，退出
                    break;
                }

                tokio::select! {
                    new_item = in_rx.recv(), if !in_closed => {
                        if let Some((item,dur)) = new_item {
                            dq.insert(item,dur);
                            empty = false;
                        } else{
                            // in_tx 关掉了
                            in_closed = true;
                        }
                    },
                    delay_item = dq.next(), if !empty => {
                        // dq 为空的时候，next()会一直返回none，在这种情况下，disable这个分支
                        if let Some(item) = delay_item {
                            // 检查out_rx通道是否关闭
                             if out_tx.send(Ok(item)).await.is_err() {
                                out_closed = true;
                            }
                        }else{
                            // dq 为空
                            empty = true;
                        }
                    }
                }
            }
        });

        (in_tx, out_rx)
    }
}
