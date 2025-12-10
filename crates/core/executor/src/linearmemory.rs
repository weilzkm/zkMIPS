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
    pub unit_table: WordMemory<T>,
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
        Self { registers: Registers::default(), unit_table: WordMemory::default() }
    }
}

impl<T: Copy> Memory<T> {
    /// Initialize a new memory with preallocated page table.
    pub fn new_preallocated() -> Self {
        Self { registers: Registers::default(), unit_table: WordMemory::new_preallocated() }
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
    #[inline]
    pub fn or_insert(&mut self, addr: u32, value: T) {
        let option = self.registers[addr as usize];
        if option.is_none() {
            self.registers[addr as usize] = Some(value);
        }
    }

    /// Insert a value into the registers.
    ///
    /// Assumes addr < NUM_REGISTERS.
    #[inline]
    pub fn insert(&mut self, addr: u32, value: T) -> Option<T> {
        self.registers[addr as usize].replace(value)
    }

    #[inline]
    pub fn insert_mut(&mut self, addr: u32, value: T) -> &mut T {
        self.registers[addr as usize] = Some(value);
        self.registers[addr as usize].as_mut().unwrap()
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

const MAX_WORD_COUNT: usize = MAX_MEMORY / 4 ;

/// Paged memory. Balances both memory locality and total memory usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "V: Serialize"))]
#[serde(bound(deserialize = "V: DeserializeOwned"))]
pub struct WordMemory<V: Copy> {
    /// The internal word memory.
    pub unit_table: Vec<Option<V>>,
}

impl<V: Copy> WordMemory<V> {

    /// Create a `WordMemory` with capacity `MAX_WORD_COUNT`.
    pub fn new_preallocated() -> Self {
        Self { unit_table: vec![None; MAX_WORD_COUNT] }
    }

    /// Get a reference to the memory value at the given address, if it exists.
    pub fn get(&self, addr: u32) -> Option<&V> {
        self.unit_table[addr as usize >> 2].as_ref()
    }

    /// Get a mutable reference to the memory value at the given address, if it exists.
    pub fn get_mut(&mut self, addr: u32) -> Option<&mut V> {
        self.unit_table[addr as usize >> 2].as_mut()
    }

    /// Insert a value at the given address. Returns the previous value, if any.
    #[inline]
    pub fn insert(&mut self, addr: u32, value: V) -> Option<V> {
        self.unit_table[addr as usize >> 2].replace(value)
    }

    /// Insert a value at the given address. Returns the previous value, if any.
    #[inline]
    pub fn insert_mut(&mut self, addr: u32, value: V) -> & mut V {
        self.unit_table[addr as usize >> 2].replace(value);
        self.unit_table[addr as usize >> 2].as_mut().unwrap()
    }

    /// Remove the value at the given address if it exists, returning it.
    pub fn remove(&mut self, addr: u32) -> Option<V> {
        self.unit_table[addr as usize >> 2].take()
    }

    pub fn or_insert(&mut self, addr: u32, value: V){
        let entry = &mut self.unit_table[addr as usize >> 2];
        if entry.is_none() {
            self.unit_table[addr as usize >> 2].replace(value);
        }
    }

    /// Returns an iterator over the occupied addresses.
    pub fn keys(&self) -> impl Iterator<Item = u32> + '_ {
        self.unit_table
            .iter()
            .enumerate()
            .filter(|(_, value)| value.is_some())
            .map(|(addr, _)| (addr << 2) as u32)
    }

    /// Get the exact number of addresses in use. This function iterates through each word
    /// and is therefore somewhat expensive.
    pub fn exact_len(&self) -> usize {
        self.unit_table
            .iter()
            .filter(|&&i| i.is_some())
            .count()
    }

    /// Estimate the number of addresses in use.
    pub fn estimate_len(&self) -> usize {
        self.unit_table.iter().filter(|&&i| i.is_some()).count()
    }

    /// Clears the page table. Drops all `Page`s, but retains the memory used by the table itself.
    pub fn clear(&mut self) {
        self.unit_table.fill(None);
    }
}

impl<V: Copy> Default for WordMemory<V> {
    fn default() -> Self {
        Self { unit_table: vec![None; MAX_WORD_COUNT] }
    }
}

impl<V: Copy> FromIterator<(u32, V)> for WordMemory<V> {
    fn from_iter<T: IntoIterator<Item = (u32, V)>>(iter: T) -> Self {
        let mut mmu = Self::new_preallocated();
        for (k, v) in iter {
            mmu.insert(k, v);
        }
        mmu
    }
}

impl<V: Copy + 'static> IntoIterator for WordMemory<V> {
    type Item = (u32, V);

    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(
            self.unit_table
                .into_iter()
                .enumerate()
                .filter(|(_, v)| v.is_some())
                .map(move |(addr, v)| ((addr << 2) as u32, v.unwrap()))
        )
    }
}
