use super::program::MAX_MEMORY;
use crate::register::NUM_REGISTERS;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use vec_map::VecMap;

/// A memory.
///
/// Consists of registers, as well as a page table for main memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "T: Serialize"))]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
pub struct Memory<T: Copy> {
    /// The registers.
    pub registers: Registers<T>,
    /// The page table.
    pub unit_table: PagedMemory<T>,
}

impl<V: Copy + 'static> IntoIterator for Memory<V> {
    type Item = (u32, V);

    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.registers.into_iter().chain(self.unit_table))
    }
}

impl<T: Copy + Default> Default for Memory<T> {
    fn default() -> Self {
        Self { registers: Registers::default(), unit_table: PagedMemory::default() }
    }
}

impl<T: Copy> Memory<T> {
    /// Initialize a new memory with preallocated page table.
    pub fn new_preallocated() -> Self {
        Self { registers: Registers::default(), unit_table: PagedMemory::new_preallocated() }
    }

    /// Insert a value into the memory.
    ///
    /// When possible, prefer directly accessing the `unit_table` or `registers` fields.
    /// This method often incurs unnecessary branching.   
    #[inline]
    pub fn insert(&mut self, addr: u32, value: T) -> Option<T> {
        if addr < NUM_REGISTERS as u32 {
            self.registers.insert(addr, value)
        } else {
            self.unit_table.insert(addr, value)
        }
    }

    /// Get a value from the memory.
    ///
    /// When possible, prefer directly accessing the `unit_table` or `registers` fields.
    /// This method often incurs unnecessary branching.
    #[inline]
    pub fn get(&self, addr: u32) -> Option<&T> {
        if addr < NUM_REGISTERS as u32 {
            self.registers.get(addr)
        } else {
            self.unit_table.get(addr)
        }
    }

    /// Remove a value from the memory.
    ///
    /// When possible, prefer directly accessing the `unit_table` or `registers` fields.
    /// This method often incurs unnecessary branching.
    #[inline]
    pub fn remove(&mut self, addr: u32) -> Option<T> {
        if addr < NUM_REGISTERS as u32 {
            self.registers.remove(addr)
        } else {
            self.unit_table.remove(addr)
        }
    }

    /// Clear the memory.
    #[inline]
    pub fn clear(&mut self) {
        self.registers.clear();
        self.unit_table.clear();
    }
}

impl<V: Copy + Default> FromIterator<(u32, V)> for Memory<V> {
    fn from_iter<T: IntoIterator<Item = (u32, V)>>(iter: T) -> Self {
        let mut memory = Self::new_preallocated();
        for (addr, value) in iter {
            memory.insert(addr, value);
        }
        memory
    }
}

/// An array of NUM_REGISTERS registers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "T: Serialize"))]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
pub struct Registers<T: Copy> {
    pub registers: Vec<Option<T>>,
}

impl<T: Copy> Default for Registers<T> {
    fn default() -> Self {
        Self { registers: vec![None; NUM_REGISTERS] }
    }
}

impl<T: Copy> Registers<T> {
    /// Insert a value into the registers.
    ///
    /// Assumes addr < NUM_REGISTERS.
    pub fn insert(&mut self, addr: u32, value: T) -> Option<T> {
        self.registers[addr as usize].replace(value)
    }

    #[inline]
    pub fn insert_mut(&mut self, addr: u32, value: T) -> &mut T {
        self.registers[addr as usize] = Some(value);
        self.registers[addr as usize].as_mut().unwrap()
    }

    #[inline]
    pub fn or_insert(&mut self, addr: u32, value: T) {
        let option = self.registers[addr as usize];
        if option.is_none() {
            self.registers[addr as usize] = Some(value);
        }
    }

    /// Remove a value from the registers, and return it if it exists.
    ///
    /// Assumes addr < NUM_REGISTERS.
    #[inline]
    pub fn remove(&mut self, addr: u32) -> Option<T> {
        self.registers[addr as usize].take()
    }

    /// Get a reference to the value at the given address, if it exists.
    ///
    /// Assumes addr < NUM_REGISTERS.
    #[inline]
    pub fn get(&self, addr: u32) -> Option<&T> {
        self.registers[addr as usize].as_ref()
    }

    #[inline]
    pub fn get_mut(&mut self, addr: u32) -> Option<&mut T> {
        self.registers[addr as usize].as_mut()
    }

    /// Clear the registers.
    #[inline]
    pub fn clear(&mut self) {
        self.registers.fill(None);
    }
}

impl<V: Copy> FromIterator<(u32, V)> for Registers<V> {
    fn from_iter<T: IntoIterator<Item = (u32, V)>>(iter: T) -> Self {
        let mut mmu = Self::default();
        for (k, v) in iter {
            mmu.insert(k, v);
        }
        mmu
    }
}

impl<V: Copy + 'static> IntoIterator for Registers<V> {
    type Item = (u32, V);

    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(
            self.registers
                .into_iter()
                .enumerate()
                .filter_map(move |(i, v)| v.map(|v| (i as u32, v))),
        )
    }
}

/// A page of memory.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<V>(VecMap<V>);

impl<V> Default for Page<V> {
    fn default() -> Self {
        Self(VecMap::default())
    }
}

const LOG_PAGE_LEN: usize = 14;
const PAGE_LEN: usize = 1 << LOG_PAGE_LEN;
const MAX_PAGE_COUNT: usize = MAX_MEMORY / 4 / PAGE_LEN + 1;
const NO_PAGE: u16 = u16::MAX;
const PAGE_MASK: usize = PAGE_LEN - 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "V: Serialize"))]
#[serde(bound(deserialize = "V: DeserializeOwned"))]
pub struct NewPage<V>(Vec<Option<V>>);

impl<V: Copy> NewPage<V> {
    pub fn new() -> Self {
        Self(vec![None; PAGE_LEN])
    }
}

impl<V: Copy> Default for NewPage<V> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

/// Paged memory. Balances both memory locality and total memory usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "V: Serialize"))]
#[serde(bound(deserialize = "V: DeserializeOwned"))]
pub struct PagedMemory<V: Copy> {
    /// The internal page table.
    pub unit_table: Vec<NewPage<V>>,
    pub index: Vec<u16>,
}

impl<V: Copy> PagedMemory<V> {
    /// The number of lower bits to ignore, since addresses (except registers) are a multiple of 4.
    const NUM_IGNORED_LOWER_BITS: usize = 2;

    /// Create a `PagedMemory` with capacity `MAX_PAGE_COUNT`.
    pub fn new_preallocated() -> Self {
        Self { unit_table: Vec::new(), index: vec![NO_PAGE; MAX_PAGE_COUNT] }
    }

    /// Get a reference to the memory value at the given address, if it exists.
    pub fn get(&self, addr: u32) -> Option<&V> {
        let (upper, lower) = Self::indices(addr);
        let index = self.index[upper];
        if index == NO_PAGE {
            None
        } else {
            self.unit_table[index as usize].0[lower].as_ref()
        }
    }

    /// Get a mutable reference to the memory value at the given address, if it exists.
    pub fn get_mut(&mut self, addr: u32) -> Option<&mut V> {
        let (upper, lower) = Self::indices(addr);
        let index = self.index[upper];
        if index == NO_PAGE {
            None
        } else {
            self.unit_table[index as usize].0[lower].as_mut()
        }
    }

    /// Insert a value at the given address. Returns the previous value, if any.
    #[inline]
    pub fn insert(&mut self, addr: u32, value: V) -> Option<V> {
        let (upper, lower) = Self::indices(addr);
        let mut index = self.index[upper];
        if index == NO_PAGE {
            index = self.unit_table.len() as u16;
            self.index[upper] = index;
            self.unit_table.push(NewPage::new());
        }
        self.unit_table[index as usize].0[lower].replace(value)
    }

    /// Insert a value at the given address. Returns the , if any.
    #[inline]
    pub fn insert_mut(&mut self, addr: u32, value: V) -> &mut V {
        let (upper, lower) = Self::indices(addr);
        let mut index = self.index[upper];
        if index == NO_PAGE {
            index = self.unit_table.len() as u16;
            self.index[upper] = index;
            self.unit_table.push(NewPage::new());
        }
        self.unit_table[index as usize].0[lower].replace(value);
        self.unit_table[index as usize].0[lower].as_mut().unwrap()
    }

    pub fn or_insert(&mut self, addr: u32, value: V) {
        let (upper, lower) = Self::indices(addr);
        let mut index = self.index[upper];
        if index == NO_PAGE {
            index = self.unit_table.len() as u16;
            self.index[upper] = index;
            self.unit_table.push(NewPage::new());
            self.unit_table[index as usize].0[lower].replace(value);
        } else {
            let option = self.unit_table[index as usize].0[lower];
            if option.is_none() {
                self.unit_table[index as usize].0[lower].replace(value);
            }
        }
    }

    /// Remove the value at the given address if it exists, returning it.
    pub fn remove(&mut self, addr: u32) -> Option<V> {
        let (upper, lower) = Self::indices(addr);
        let index = self.index[upper];
        if index == NO_PAGE {
            None
        } else {
            self.unit_table[index as usize].0[lower].take()
        }
    }

    /// Returns an iterator over the occupied addresses.
    pub fn keys(&self) -> impl Iterator<Item = u32> + '_ {
        self.index.iter().enumerate().filter(|(_, &i)| i != NO_PAGE).flat_map(|(i, index)| {
            let upper = i << LOG_PAGE_LEN;
            self.unit_table[*index as usize]
                .0
                .iter()
                .enumerate()
                .filter_map(move |(lower, v)| v.map(|_| Self::decompress_addr(upper + lower)))
        })
    }

    /// Get the exact number of addresses in use. This function iterates through each page
    /// and is therefore somewhat expensive.
    pub fn exact_len(&self) -> usize {
        self.index
            .iter()
            .filter(|&&i| i != NO_PAGE)
            .map(|index| self.unit_table[*index as usize].0.iter().filter(|v| v.is_some()).count())
            .sum()
    }

    /// Estimate the number of addresses in use.
    pub fn estimate_len(&self) -> usize {
        self.index.iter().filter(|&i| *i != NO_PAGE).count() * PAGE_LEN
    }

    /// Clears the page table. Drops all `Page`s, but retains the memory used by the table itself.
    pub fn clear(&mut self) {
        self.unit_table.clear();
        self.index.fill(NO_PAGE);
    }

    /// Break apart an address into an upper and lower index.
    #[inline]
    const fn indices(addr: u32) -> (usize, usize) {
        let index = Self::compress_addr(addr);
        (index >> LOG_PAGE_LEN, index & PAGE_MASK)
    }

    /// Compress an address from the sparse address space to a contiguous space.
    #[inline]
    const fn compress_addr(addr: u32) -> usize {
        addr as usize >> Self::NUM_IGNORED_LOWER_BITS
    }

    /// Decompress an address from a contiguous space to the sparse address space.
    #[inline]
    const fn decompress_addr(addr: usize) -> u32 {
        (addr << Self::NUM_IGNORED_LOWER_BITS) as u32
    }
}

impl<V: Copy> Default for PagedMemory<V> {
    fn default() -> Self {
        Self { unit_table: Vec::new(), index: vec![NO_PAGE; MAX_PAGE_COUNT] }
    }
}

impl<V: Copy> FromIterator<(u32, V)> for PagedMemory<V> {
    fn from_iter<T: IntoIterator<Item = (u32, V)>>(iter: T) -> Self {
        let mut mmu = Self::new_preallocated();
        for (k, v) in iter {
            mmu.insert(k, v);
        }
        mmu
    }
}

impl<V: Copy + 'static> IntoIterator for PagedMemory<V> {
    type Item = (u32, V);

    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(mut self) -> Self::IntoIter {
        Box::new(self.index.into_iter().enumerate().filter(|(_, i)| *i != NO_PAGE).flat_map(
            move |(i, index)| {
                let upper = i << LOG_PAGE_LEN;
                std::mem::take(&mut self.unit_table[index as usize])
                    .0
                    .into_iter()
                    .enumerate()
                    .filter_map(move |(lower, v)| {
                        v.map(|v| (Self::decompress_addr(upper + lower), v))
                    })
            },
        ))
    }
}
