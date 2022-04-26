/// Defines events that Hub deals with

use libp2p::identify::IdentifyEvent;
use libp2p::ping::PingEvent;
use libp2p::relay::v2::relay::Event as RelayEvent;
use libp2p::autonat::Event as AutoNatEvent;

#[derive(Debug)]
pub enum Event {
    Ping(PingEvent),
    Identify(IdentifyEvent),
    Relay(RelayEvent),
    AutoNat(AutoNatEvent),
}

impl From<PingEvent> for Event {
    fn from(e: PingEvent) -> Self {
        Event::Ping(e)
    }
}

impl From<IdentifyEvent> for Event {
    fn from(e: IdentifyEvent) -> Self {
        Event::Identify(e)
    }
}

impl From<RelayEvent> for Event {
    fn from(e: RelayEvent) -> Self {
        Event::Relay(e)
    }
}

impl From<AutoNatEvent> for Event {
    fn from(e: AutoNatEvent) -> Self {
        Event::AutoNat(e)
    }
}
