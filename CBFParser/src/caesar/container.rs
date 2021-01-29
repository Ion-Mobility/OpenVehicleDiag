use std::sync::Arc;

use common::raf::Raf;
use ctf_header::CTFHeader;
use ecu::ECU;

use crate::{ctf::{STUB_HEADER_SIZE, StubHeader, cff_header::CFFHeader, ctf_header}, ecu};



#[derive(Debug, Clone, Default)]
pub(crate) struct Container {
    cff_header: CFFHeader,
    ctf_header: CTFHeader,
    pub ecus: Vec<ECU>
}

impl Container {
    pub fn new(reader: &mut Raf) -> super::Result<Self> {
        reader.seek(0);

        let header = reader.read_bytes(STUB_HEADER_SIZE)?;
        StubHeader::read_header(&header);

        let cff_header_size = reader.read_i32()? as usize;
        let cff_header_bytes = reader.read_bytes(cff_header_size)?;

        let mut container = Container::default();

        container.cff_header = container.read_cff(reader)?;
        container.ctf_header = container.read_ctf(reader)?;

        match container.read_ecus(reader) {
            Ok(l) => container.ecus = l,
            Err(e) => {
                eprintln!("Error reading ECUs: {:?}", e)
            }
        }
        Ok(container)
    }

    fn read_cff(&self, reader: &mut Raf) -> super::Result<CFFHeader> {
        CFFHeader::new(reader)
    }

    fn read_ctf(&self, reader: &mut Raf) -> super::Result<CTFHeader> {
        let offset = self.cff_header.base_addr + self.cff_header.ctf_offset as usize;
        CTFHeader::new(reader, offset, self.cff_header.cff_header_size as usize)
    }

    fn read_ecus(&mut self, reader: &mut Raf) -> super::Result<Vec<ECU>> {
        let ecu_table_offset = self.cff_header.ecu_offset as usize + self.cff_header.base_addr;

        let mut ecus = Vec::new();
        for i in 0..self.cff_header.ecu_count as usize {
            let arc = Arc::new(self.clone());
            reader.seek(ecu_table_offset + (i*4));
            let offset_to_actual_ecu = reader.read_i32()? as usize;
            ecus.push(ECU::new(reader, &self.ctf_header.get_languages(0), &self.cff_header,ecu_table_offset + offset_to_actual_ecu, arc)?)
        }
        Ok(ecus)
    }
}