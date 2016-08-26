extern crate lossyq;
extern crate parking_lot;
extern crate time;
extern crate libc;

pub mod scheduler;
pub mod elem;

use lossyq::spsc::Receiver;
use std::fmt;

#[derive(Copy,Clone,Debug)]
pub enum Message<T: Send>
{
  Empty,                      //
  Value(T),                   //
  Ack(usize,usize),           // from-to
  Error(usize,&'static str),  // error at
}

#[derive(Copy,Clone,Debug)]
pub enum Schedule {
  Loop,
  OnMessage(usize, usize), // channel id, msg id
  DelayUSec(u64),
  OnExternalEvent,
  Stop,
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub enum TaskState {
  Execute,
  TimeWait(usize),
  MessageWait(usize, usize), // ch_id, msg_id
  ExtEventWait(usize),
  Stop,
}

#[derive(Copy,Clone,Debug)]
pub enum Error {
  Busy,
  NonExistent,
  Stopping,
  AlreadyExists,
}

#[derive(Clone,Debug)]
pub struct ChannelId {
  task_name  : String,
  id         : usize,
}

pub trait Observer {
  fn scheduled(&mut self, task_id: usize, at_usec: usize);
  fn executed(&mut self, task_id: usize, at_usec: usize);
  fn stopped(&mut self, task_id: usize, at_usec: usize);
  fn delayed(&mut self, task_id: usize, reason: &TaskState, at_usec: usize);
  fn message_sent(&mut self, channel_id: usize, last_msg_id: usize, task_id: usize, at_usec: usize);
  fn wait_channel(&mut self, channel_id: &ChannelId, last_msg_id: usize, task_id: usize, at_usec: usize);
  fn transition(&mut self,
    from: &TaskState,
    event: &Schedule,
    to: &TaskState,
    task_id: usize,
    at_usec: usize);
}

pub trait Task {
  fn execute(&mut self) -> Schedule;
  fn name(&self)  -> &String;
  fn input_count(&self) -> usize;
  fn output_count(&self) -> usize;
  fn input_id(&self, ch_id: usize) -> Option<ChannelId>;

  fn output_id(&self, ch_id: usize) -> Option<ChannelId> {
    if ch_id >= self.output_count() {
      None
    } else {
      Some( new_id(self.name().clone(), ch_id))
    }
  }
  fn tx_count(&self, ch_id: usize) -> usize;
}

pub struct IdentifiedReceiver<Input: Send> {
  pub id    : ChannelId,
  pub input : Receiver<Message<Input>>,
}

impl fmt::Display for ChannelId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Id:({} {})", self.task_name, self.id)
    }
}

pub fn new_id(name: String, id: usize) -> ChannelId {
  ChannelId {
    task_name  : name,
    id         : id,
  }
}


#[cfg(test)]
pub mod tests;
