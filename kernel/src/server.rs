pub use crate::arch::ProcessContext;
use core::{mem, slice};
use xous::{MemoryAddress, MemorySize, PID, SID};

/// Internal representation of a queued message for a server.
/// This should be exactly 8 words / 32 bytes, yielding 128
/// queued messages per server
#[repr(usize)]
#[derive(PartialEq)]
enum QueuedMessage {
    Empty,
    ScalarMessage(
        usize, /* sender */
        usize, /* context */
        usize, /* id */
        usize, /* arg1 */
        usize, /* arg2 */
        usize, /* arg3 */
        usize, /* arg4 */
    ),
    MemoryMessageSend(
        usize, /* sender */
        usize, /* context */
        usize, /* id */
        usize, /* buf */
        usize, /* buf_size */
        usize, /* offset */
        usize, /* valid */
    ),
    MemoryMessageROLend(
        usize, /* sender */
        usize, /* context */
        usize, /* id */
        usize, /* buf */
        usize, /* buf_size */
        usize, /* offset */
        usize, /* valid */
    ),
    MemoryMessageRWLend(
        usize, /* sender */
        usize, /* context */
        usize, /* id */
        usize, /* buf */
        usize, /* buf_size */
        usize, /* offset */
        usize, /* valid */
    ),
}

/// A pointer to resolve a server ID to a particular process
#[derive(PartialEq)]
pub struct Server {
    /// A randomly-generated ID
    pub sid: SID,

    /// The process that owns this server
    pub pid: PID,

    /// An index into the queue
    queue_head: usize,

    queue_tail: usize,

    /// Where data will appear
    queue: &'static mut [QueuedMessage],

    /// The `context mask` is a bitfield of contexts that are able to handle
    /// this message. If there are no available contexts, then messages will
    /// need to be queued.
    ready_contexts: usize,
}

impl Server {
    pub fn init(
        new: &mut Option<Server>,
        pid: PID,
        sid: SID,
        queue_addr: *mut usize,
        queue_size: usize,
    ) -> Result<(), xous::Error> {
        if new != &None {
            return Err(xous::Error::MemoryInUse);
        }

        let queue = unsafe {
            slice::from_raw_parts_mut(
                queue_addr as *mut QueuedMessage,
                queue_size / mem::size_of::<QueuedMessage>(),
            )
        };

        *new = Some(Server {
            sid,
            pid,
            queue_head: 0,
            queue_tail: 0,
            queue,
            ready_contexts: 0,
        });
        Ok(())
    }
    /// Remove a message from the server's queue and replace it with
    /// QueuedMessage::Empty. Advance the queue pointer while we're at it.
    pub fn take_next_message(&mut self) -> Option<(xous::MessageEnvelope, usize)> {
        let result = match self.queue[self.queue_head] {
            QueuedMessage::Empty => return None,
            QueuedMessage::MemoryMessageROLend(
                sender,
                context,
                id,
                buf,
                buf_size,
                offset,
                valid,
            ) => (
                xous::MessageEnvelope {
                    sender: sender,
                    message: xous::Message::ImmutableBorrow(xous::MemoryMessage {
                        id,
                        buf: MemoryAddress::new(buf),
                        buf_size: MemorySize::new(buf_size),
                        _offset: MemorySize::new(offset),
                        _valid: MemorySize::new(valid),
                    }),
                },
                context,
            ),
            QueuedMessage::MemoryMessageRWLend(
                sender,
                context,
                id,
                buf,
                buf_size,
                offset,
                valid,
            ) => (
                xous::MessageEnvelope {
                    sender: sender,
                    message: xous::Message::MutableBorrow(xous::MemoryMessage {
                        id,
                        buf: MemoryAddress::new(buf),
                        buf_size: MemorySize::new(buf_size),
                        _offset: MemorySize::new(offset),
                        _valid: MemorySize::new(valid),
                    }),
                },
                context,
            ),
            QueuedMessage::MemoryMessageSend(sender, context, id, buf, buf_size, offset, valid) => {
                (
                    xous::MessageEnvelope {
                        sender: sender,
                        message: xous::Message::Move(xous::MemoryMessage {
                            id,
                            buf: MemoryAddress::new(buf),
                            buf_size: MemorySize::new(buf_size),
                            _offset: MemorySize::new(offset),
                            _valid: MemorySize::new(valid),
                        }),
                    },
                    context,
                )
            }
            QueuedMessage::ScalarMessage(sender, context, id, arg1, arg2, arg3, arg4) => (
                xous::MessageEnvelope {
                    sender: sender,
                    message: xous::Message::Scalar(xous::ScalarMessage {
                        id,
                        arg1,
                        arg2,
                        arg3,
                        arg4,
                    }),
                },
                context,
            ),
        };
        self.queue[self.queue_head] = QueuedMessage::Empty;
        self.queue_head += 1;
        if self.queue_head >= self.queue.len() {
            self.queue_head = 0;
        }
        Some(result)
    }

    /// Add the given message to this server's queue.
    pub fn queue_message(
        &mut self,
        envelope: xous::MessageEnvelope,
        context: usize,
    ) -> core::result::Result<(), xous::Error> {
        if self.queue[self.queue_head] != QueuedMessage::Empty {
            return Err(xous::Error::ServerQueueFull);
        }

        self.queue_head += 1;
        if self.queue_head >= self.queue.len() {
            self.queue_head = 0;
        }
        Ok(())
    }

    // assert!(
    //     mem::size_of::<QueuedMessage>() == 32,
    //     "QueuedMessage was supposed to be 32 bytes, but instead was {} bytes",
    //     mem::size_of::<QueuedMessage>()
    // );

    /// Return a context ID that is available and blocking.  If no such context ID exists,
    /// or if this server isn't actually ready to receive packets, return None.
    pub fn take_available_context(&mut self) -> Option<usize> {
        if self.ready_contexts == 0 {
            return None;
        }
        let mut test_ctx_mask = 1;
        let mut ctx_number = 0;
        loop {
            // If the context mask matches this context number, remove it
            // and return the index.
            if self.ready_contexts & test_ctx_mask == test_ctx_mask {
                self.ready_contexts = self.ready_contexts & !test_ctx_mask;
                return Some(ctx_number);
            }
            // Advance to the next slot.
            test_ctx_mask = test_ctx_mask.rotate_left(1);
            ctx_number = ctx_number + 1;
            if test_ctx_mask == 1 {
                panic!("didn't find a free context, even though there should be one");
            }
        }
    }

    pub fn park_context(&mut self, context: usize) {
        self.ready_contexts |= 1 << context;
    }
}