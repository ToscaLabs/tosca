use alloc::format;

use tosca::events::Topic;

pub(crate) struct TopicBuilder<'a> {
    prefix: &'a str,
    mac: [u8; 6],
    suffix: &'a str,
}

impl<'a> TopicBuilder<'a> {
    pub(crate) const fn new() -> Self {
        Self {
            prefix: "",
            mac: [0; 6],
            suffix: "",
        }
    }

    pub(crate) const fn prefix(mut self, prefix: &'a str) -> Self {
        self.prefix = prefix;
        self
    }

    pub(crate) const fn mac(mut self, mac: [u8; 6]) -> Self {
        self.mac = mac;
        self
    }

    pub(crate) const fn suffix(mut self, suffix: &'a str) -> Self {
        self.suffix = suffix;
        self
    }

    #[inline]
    pub(crate) fn build(self) -> Topic {
        Topic::new(format!(
            "{}/{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}/{}",
            self.prefix,
            self.mac[0],
            self.mac[1],
            self.mac[2],
            self.mac[3],
            self.mac[4],
            self.mac[5],
            self.suffix
        ))
    }
}
