use std::net::{Ipv4Addr, Ipv6Addr};

pub struct PublicIp {
    v4: Option<Ipv4Addr>,
    v6: Option<Ipv6Addr>,
}

impl PublicIp {
    pub fn new(v4: Ipv4Addr, v6: Ipv6Addr) -> Self {
        Self {
            v4: Some(v4),
            v6: Some(v6),
        }
    }

    pub fn new_v4(v4: Ipv4Addr) -> Self {
        Self {
            v4: Some(v4),
            v6: None,
        }
    }

    pub fn new_v6(v6: Ipv6Addr) -> Self {
        Self {
            v4: None,
            v6: Some(v6),
        }
    }

    pub fn set_v4(&mut self, v4: Ipv4Addr) {
        self.v4 = Some(v4);
    }

    pub fn set_v6(&mut self, v6: Ipv6Addr) {
        self.v6 = Some(v6);
    }

    pub fn get_v4(&self) -> Option<&Ipv4Addr> {
        self.v4.as_ref()
    }

    pub fn get_v6(&self) -> Option<&Ipv6Addr> {
        self.v6.as_ref()
    }
}
