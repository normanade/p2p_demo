/// Defines events that Hub deals with

use libp2p::identify::IdentifyEvent;
use libp2p::ping::PingEvent;
use libp2p::relay::v2::relay::Event as RelayEvent;
use libp2p::relay::v2::client::Event as RelayClientEvent;
use libp2p::dcutr::behaviour::Event as DcutrEvent;

#[derive(Debug)]
pub enum Event {
    Ping(PingEvent),
    Identify(IdentifyEvent),
    Relay(RelayEvent),
    RelayClient(RelayClientEvent),
    Dcutr(DcutrEvent),
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

impl From<RelayClientEvent> for Event {
    fn from(e: RelayClientEvent) -> Self {
        Event::RelayClient(e)
    }
}

impl From<DcutrEvent> for Event {
    fn from(e: DcutrEvent) -> Self {
        Event::Dcutr(e)
    }
}
