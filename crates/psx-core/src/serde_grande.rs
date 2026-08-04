use std::cell::Cell;
use std::fmt;
use std::marker::PhantomData;

use serde::de::{Error, SeqAccess, Visitor};
use serde::ser::SerializeTuple;
use serde::{Deserializer, Serializer};

// serde so implementa Serialize/Deserialize para `[T; N]` ate N=32. VRAM, tabelas do MDEC,
// registradores do GTE e o buffer de setor do CD-ROM passam disso.
struct ArrayVisitor<T, const N: usize>(PhantomData<T>);

impl<'de, T, const N: usize> Visitor<'de> for ArrayVisitor<T, N>
where
    T: serde::Deserialize<'de> + Default + Copy,
{
    type Value = [T; N];

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "um array de {N} elementos")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<[T; N], A::Error> {
        let mut saida = [T::default(); N];
        for (i, lugar) in saida.iter_mut().enumerate() {
            *lugar = seq
                .next_element()?
                .ok_or_else(|| A::Error::invalid_length(i, &self))?;
        }
        Ok(saida)
    }
}

pub fn serialize<S, T, const N: usize>(valor: &[T; N], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: serde::Serialize,
{
    let mut tupla = s.serialize_tuple(N)?;
    for elemento in valor {
        tupla.serialize_element(elemento)?;
    }
    tupla.end()
}

pub fn deserialize<'de, D, T, const N: usize>(d: D) -> Result<[T; N], D::Error>
where
    D: Deserializer<'de>,
    T: serde::Deserialize<'de> + Default + Copy,
{
    d.deserialize_tuple(N, ArrayVisitor::<T, N>(PhantomData))
}

pub mod em_cell {
    use super::{Cell, Deserializer, Serializer};

    pub fn serialize<S, T, const N: usize>(valor: &Cell<[T; N]>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: serde::Serialize + Default + Copy,
    {
        super::serialize(&valor.get(), s)
    }

    pub fn deserialize<'de, D, T, const N: usize>(d: D) -> Result<Cell<[T; N]>, D::Error>
    where
        D: Deserializer<'de>,
        T: serde::Deserialize<'de> + Default + Copy,
    {
        super::deserialize(d).map(Cell::new)
    }
}
