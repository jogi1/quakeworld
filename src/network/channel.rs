
use serde::Serialize;

#[derive(Default, PartialEq, Eq, Serialize)]
pub enum ReliableState {
    #[default] None,
    Send,
    Ack,
    Recieved,
}

#[derive(Default, PartialEq, Eq, Serialize)]
pub struct Sequence {
    pub sequence: u32,
    pub last_reliable: u32,
    pub reliable_state: ReliableState,
}

#[derive(Default, Serialize)]
pub struct Channel {
    pub outgoing: Sequence,
    pub acknowledged: Sequence,
}

impl Channel {
    pub fn can_reliable(&self) -> bool {
        if self.outgoing.reliable_state == ReliableState::Send {
            return false;
        }
        true
    }

    pub fn reliable(&mut self) -> (u32,u32) {
        self.outgoing.sequence += 1;
        self.outgoing.last_reliable = self.outgoing.sequence;
        self.outgoing.reliable_state = ReliableState::Send;
        let mut ack_rel = 0;
        if self.acknowledged.reliable_state == ReliableState::Recieved {
            ack_rel = 1;
            self.acknowledged.reliable_state = ReliableState::Send;
        }
        (self.outgoing.sequence | (1 << 31), self.acknowledged.sequence | (ack_rel << 31)) 
    }

    pub fn unreliable(&mut self) -> (u32,u32) {
        self.outgoing.sequence += 1;
        let mut ack_rel = 0;
        if self.acknowledged.reliable_state == ReliableState::Recieved {
            ack_rel = 1;
            self.acknowledged.reliable_state = ReliableState::Send;
        }
        (self.outgoing.sequence , self.acknowledged.sequence | (ack_rel << 31))
    }

    pub fn recieved(&mut self, sequence_in: u32, acknowledged_in: u32) -> bool {
        let sequence_reliable = sequence_in & (1 << 31) != 0;
        let sequence =  sequence_in  & !(1 << 31);
        if sequence_reliable {
            self.acknowledged.reliable_state = ReliableState::Recieved;
        }
        self.acknowledged.sequence = sequence;

        let acknowledged_reliable = acknowledged_in & (1 << 31) != 0;
        let acknowledged_sequence =  acknowledged_in & !(1 << 31);
        if acknowledged_reliable {
            if self.outgoing.last_reliable != acknowledged_sequence {
            }
            self.outgoing.last_reliable = 0;
            self.outgoing.reliable_state = ReliableState::Ack;
        }
        sequence_reliable
    }
}

