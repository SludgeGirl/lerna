use std::{iter::Enumerate, net::Ipv4Addr};

use cidr::{InetIterator, Ipv4Cidr, Ipv4Inet, errors::NetworkParseError};

const DEFAULT_IPV4_CIDR: Result<Ipv4Cidr, NetworkParseError> =
    Ipv4Cidr::new(Ipv4Addr::new(100, 64, 0, 0), 10);

#[derive(Debug)]
pub struct Ipv4Issuer {
    cidr_range: Enumerate<InetIterator<Ipv4Addr>>,
}

impl Ipv4Issuer {
    pub fn new(ip_addr: Ipv4Addr, range: u8) -> Result<Self, NetworkParseError> {
        let cidr = Ipv4Cidr::new(ip_addr, range)?;

        Ok(Self {
            cidr_range: cidr.iter().enumerate(),
        })
    }

    pub fn set_latest_ip(&mut self, latest_issued: Ipv4Addr) {
        self.position(|p| p.address() == latest_issued);
    }

    pub fn next_with_index(&mut self) -> Option<(usize, Ipv4Inet)> {
        self.cidr_range.next()
    }
}

impl Default for Ipv4Issuer {
    fn default() -> Self {
        Self {
            cidr_range: DEFAULT_IPV4_CIDR
                .expect("Default ipv4 cidr failed")
                .iter()
                .enumerate(),
        }
    }
}

impl Iterator for Ipv4Issuer {
    type Item = Ipv4Inet;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((_, ip)) = self.cidr_range.next() {
            return Some(ip);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use cidr::Ipv4Inet;

    use crate::Ipv4Issuer;

    #[test]
    fn test_cidr() {
        let mut issuer = Ipv4Issuer::default();

        assert_eq!(
            issuer.next().unwrap().address(),
            Ipv4Addr::new(100, 64, 0, 0)
        );

        assert_eq!(
            issuer.next().unwrap().address(),
            Ipv4Addr::new(100, 64, 0, 1)
        );

        assert_eq!(
            issuer.next().unwrap().address(),
            Ipv4Addr::new(100, 64, 0, 2)
        );

        issuer.set_latest_ip(Ipv4Addr::new(100, 64, 0, 200));

        assert_eq!(
            issuer.next().unwrap().address(),
            Ipv4Addr::new(100, 64, 0, 201)
        );

        assert_eq!(
            issuer.next().unwrap().address(),
            Ipv4Addr::new(100, 64, 0, 202)
        );

        issuer.set_latest_ip(Ipv4Addr::new(100, 64, 0, 255));

        assert_eq!(
            issuer.next().unwrap().address(),
            Ipv4Addr::new(100, 64, 1, 0)
        );

        assert_eq!(
            issuer.next_with_index().unwrap(),
            (
                257 as usize,
                Ipv4Inet::new(Ipv4Addr::new(100, 64, 1, 1), 10).unwrap()
            )
        );
    }
}
