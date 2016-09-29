use lossyq::spsc::{Sender, channel};
use super::super::{Task, Message, Schedule, ChannelWrapper, ChannelId,
  SenderChannelId, ReceiverChannelId, ReceiverName, SenderName, ChannelPosition
};
use super::connectable::{Connectable};
use super::identified_input::{IdentifiedInput};
use super::counter::{OutputCounter, InputCounter};

pub trait Filter {
  type InputType   : Send;
  type OutputType  : Send;

  fn process(
    &mut self,
    input:   &mut ChannelWrapper<Self::InputType>,
    output:  &mut Sender<Message<Self::OutputType>>) -> Result<(), &'static str>;
}

pub struct FilterWrap<Input: Send, Output: Send> {
  name         : String,
  state        : Box<Filter<InputType=Input,OutputType=Output>+Send>,
  input_rx     : ChannelWrapper<Input>,
  output_tx    : Sender<Message<Output>>,
}

impl<Input: Send, Output: Send> IdentifiedInput for FilterWrap<Input,Output> {
  fn get_input_id(&self, ch_id: ReceiverChannelId) -> Option<(ChannelId, SenderName)> {
    if ch_id.0 == 0 {
      match &self.input_rx {
        &ChannelWrapper::ConnectedReceiver(ref channel_id, ref _receiver, ref sender_name) => {
          Some((*channel_id, sender_name.clone()))
        },
        _ => None,
      }
    } else {
      None
    }
  }
}

impl<Input: Send, Output: Send> InputCounter for FilterWrap<Input,Output> {
  fn get_rx_count(&self, ch_id: ReceiverChannelId) -> usize {
    if ch_id.0 == 0 {
      if let &ChannelWrapper::ConnectedReceiver(ref _channel_id, ref receiver, ref _sender_name) = &self.input_rx {
        receiver.seqno()
      } else {
        0
      }
    } else {
      0
    }
  }
}

impl<Input: Send, Output: Send> OutputCounter for FilterWrap<Input,Output> {
  fn get_tx_count(&self, ch_id: SenderChannelId) -> usize {
    if ch_id.0 == 0 {
      self.output_tx.seqno()
    } else {
      0
    }
  }
}

impl<Input: Send, Output: Send> Connectable for FilterWrap<Input,Output> {
  type Input = Input;

  fn input(&mut self) -> &mut ChannelWrapper<Input> {
    &mut self.input_rx
  }
}

impl<Input: Send, Output: Send> Task for FilterWrap<Input,Output> {
  fn execute(&mut self) -> Result<(), &'static str> {
    self.state.process(&mut self.input_rx, &mut self.output_tx)
  }
  fn name(&self) -> &String { &self.name }
  fn input_count(&self) -> usize { 1 }
  fn output_count(&self) -> usize { 1 }

  fn input_id(&self, ch_id: ReceiverChannelId) -> Option<(ChannelId, SenderName)> {
    self.get_input_id(ch_id)
  }

  fn input_channel_pos(&self, ch_id: ReceiverChannelId) -> ChannelPosition {
    ChannelPosition( self.get_rx_count(ch_id) )
  }

  fn output_channel_pos(&self, ch_id: SenderChannelId) -> ChannelPosition {
    ChannelPosition( self.get_tx_count(ch_id) )
  }
}

pub fn new<Input: Send, Output: Send>(
    name            : &str,
    output_q_size   : usize,
    filter          : Box<Filter<InputType=Input,OutputType=Output>+Send>)
      -> (Box<FilterWrap<Input,Output>>, Box<ChannelWrapper<Output>>)
{
  let (output_tx, output_rx) = channel(output_q_size);
  let name = String::from(name);

  (
    Box::new(
      FilterWrap{
        name        : name.clone(),
        state       : filter,
        input_rx    : ChannelWrapper::ReceiverNotConnected(
          ReceiverChannelId(0),
          ReceiverName (name.clone())
        ),
        output_tx   : output_tx,
      }
    ),
    Box::new(
      ChannelWrapper::SenderNotConnected(
        SenderChannelId(0),
        output_rx,
        SenderName(name)
      )
    )
  )
}
