pub type Address = u128;

pub trait Bus {
    fn select_address(&mut self, addr: Address);
    fn read_u8(&mut self) -> u8;
    fn read_u16(&mut self) -> u16;
    fn read_u32(&mut self) -> u32;
    fn read_u64(&mut self) -> u64;
    fn read_u128(&mut self) -> u128;
    fn write_u8(&mut self, data: u8);
    fn write_u16(&mut self, data: u16);
    fn write_u32(&mut self, data: u32);
    fn write_u64(&mut self, data: u64);
    fn write_u128(&mut self, data: u128);
}


#[derive(Clone, Copy, Debug)]
pub enum MemoryMapEntryType {
    UnmappedLow,
    UnmappedHigh,
    Read,
    Write,
    ReadWrite,
}

type ReadU8Delegate = Option<fn(Address) -> u8>;
type ReadU16Delegate = Option<fn(Address) -> u16>;
type ReadU32Delegate = Option<fn(Address) -> u32>;
type ReadU64Delegate = Option<fn(Address) -> u64>;
type ReadU128Delegate = Option<fn(Address) -> u128>;

type WriteU8Delegate = Option<fn(Address, u8)>;
type WriteU16Delegate = Option<fn(Address, u16)>;
type WriteU32Delegate = Option<fn(Address, u32)>;
type WriteU64Delegate = Option<fn(Address, u64)>;
type WriteU128Delegate = Option<fn(Address, u128)>;

#[derive(Clone, Copy, Debug)]
pub struct MemoryMapEntry {
    start: Address,
    end: Address,
    entry_type: MemoryMapEntryType,
    func_read_u8: ReadU8Delegate,
    func_read_u16: ReadU16Delegate,
    func_read_u32: ReadU32Delegate,
    func_read_u64: ReadU64Delegate,
    func_read_u128: ReadU128Delegate,
    func_write_u8: WriteU8Delegate,
    func_write_u16: WriteU16Delegate,
    func_write_u32: WriteU32Delegate,
    func_write_u64: WriteU64Delegate,
    func_write_u128: WriteU128Delegate,
}

impl MemoryMapEntry {
    pub fn new() -> MemoryMapEntry {
        MemoryMapEntry {
            start: 0,
            end: std::u128::MAX,
            entry_type: MemoryMapEntryType::UnmappedLow,
            func_read_u8: None,
            func_read_u16: None,
            func_read_u32: None,
            func_read_u64: None,
            func_read_u128: None,
            func_write_u8: None,
            func_write_u16: None,
            func_write_u32: None,
            func_write_u64: None,
            func_write_u128: None,
        }
    }
}

pub struct MemoryMap {
    entries: Vec<MemoryMapEntry>,
    current_addr: Address,
    global_addr_mask: Address,
}

#[derive(Debug, PartialEq)]
pub enum MemoryMapError {
    NoEntriesFound { addr: Address },
}

impl MemoryMap {
    fn addr(&self) -> Address {
        self.current_addr & self.global_addr_mask
    }
}

fn search_entries(map: &mut MemoryMap) -> MemoryMapEntry {
    let mut entry_count = 0;
    for (i, entry) in map.entries.iter().enumerate() {
        if (entry.start <= map.addr()) && (entry.end >= map.addr()) {
            return *entry;
        }
        entry_count = i;
    }
    map.entries.push(MemoryMapEntry::new());
    map.entries[entry_count + 1]
}


impl MemoryMap {
    pub fn new() -> MemoryMap {
        MemoryMap {
            entries: vec![MemoryMapEntry::new(); 1],
            current_addr: 0,
            global_addr_mask: std::u128::MAX,
        }
    }
}

impl Bus for MemoryMap {
    fn select_address(&mut self, addr: Address) {
        self.current_addr = addr & self.global_addr_mask;
    }
    fn read_u8(&mut self) -> u8 {
        let entry = search_entries(self);
        match entry.func_read_u8 {
            Some(func) => func(self.current_addr),
            None => {
                match entry.entry_type {
                    MemoryMapEntryType::UnmappedLow => 0,
                    MemoryMapEntryType::UnmappedHigh => 0xff,
                    _ => panic!("Your memory map is broken. Please fix it!"),
                }
            }
        }
    }

    fn read_u16(&mut self) -> u16 {
        let entry = search_entries(self);
        match entry.func_read_u16 {
            Some(func) => func(self.current_addr),
            None => self.read_u8() as u16,
        }
    }

    fn read_u32(&mut self) -> u32 {
        let entry = search_entries(self);
        match entry.func_read_u32 {
            Some(func) => func(self.current_addr),
            None => self.read_u16() as u32,
        }
    }

    fn read_u64(&mut self) -> u64 {
        let entry = search_entries(self);
        match entry.func_read_u64 {
            Some(func) => func(self.current_addr),
            None => self.read_u32() as u64,
        }
    }

    fn read_u128(&mut self) -> u128 {
        let entry = search_entries(self);
        match entry.func_read_u128 {
            Some(func) => func(self.current_addr),
            None => self.read_u64() as u128,
        }
    }

    fn write_u8(&mut self, data: u8) {
        let entry = search_entries(self);
        match entry.func_write_u8 {
            Some(func) => func(self.current_addr, data),
            None => return,
        }
    }

    fn write_u16(&mut self, data: u16) {
        let entry = search_entries(self);
        match entry.func_write_u16 {
            Some(func) => func(self.current_addr, data),
            None => return,
        }
    }

    fn write_u32(&mut self, data: u32) {
        let entry = search_entries(self);
        match entry.func_write_u32 {
            Some(func) => func(self.current_addr, data),
            None => return,
        }
    }

    fn write_u64(&mut self, data: u64) {
        let entry = search_entries(self);
        match entry.func_write_u64 {
            Some(func) => func(self.current_addr, data),
            None => return,
        }
    }

    fn write_u128(&mut self, data: u128) {
        let entry = search_entries(self);
        match entry.func_write_u128 {
            Some(func) => func(self.current_addr, data),
            None => return,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_no_memory_map_entries() {
        let mut map = MemoryMap::new();
        map.select_address(5);
        assert!(search_entries(&mut map).func_read_u8.is_none());
    }

    #[test]
    fn test_reading_unmapped() {
        let mut map = MemoryMap::new();
        map.select_address(0);
        assert_eq!(map.read_u8(), 0);
        map.entries[0].entry_type = MemoryMapEntryType::UnmappedHigh;
        assert_eq!(map.read_u8(), 0xFF);
    }

    #[test]
    fn test_writing_unmapped() {
        let mut map = MemoryMap::new();
        map.select_address(0);
        map.entries[0].entry_type = MemoryMapEntryType::UnmappedHigh;
        map.write_u8(0x55);
        assert_eq!(map.read_u8(), 0xFF);
    }

    #[test]
    fn test_global_address_mask() {
        let mut map = MemoryMap::new();
        map.global_addr_mask = 1;
        map.select_address(5);
        assert_eq!(map.current_addr, 1);
        map.global_addr_mask = 7;
        map.select_address(5);
        assert_eq!(map.current_addr, 5);
        map.select_address(8);
        assert_eq!(map.current_addr, 0);
    }
}