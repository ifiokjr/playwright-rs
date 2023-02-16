use crate::imp::{
    core::{Guid, OnlyGuid},
    prelude::*
};
use serde::ser;
use std::{cell::RefCell, mem, rc::Rc};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0:}")]
    Msg(String),
    #[error("Couldn't construct map from odd number of values")]
    OddMap,
    #[error("Key must be string")]
    InvalidKey,
    #[error("Not supported")]
    NotSupported,
    #[error("Failed to serialize JsHandle")]
    JsHandle,
    #[error("Failed to serialize DateTime")]
    DateTime,
    #[error(transparent)]
    Serde(#[from] serde_json::error::Error)
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display
    {
        Self::Msg(msg.to_string())
    }
}

#[derive(Clone, Default)]
pub(crate) struct Serializer {
    handles: Rc<RefCell<Vec<OnlyGuid>>>,

    seq: Vec<Seq>,
    t: Vec<TupleVariant>,
    om: Vec<ObjectM>,
    os: Vec<ObjectS>,
    s: Vec<StructVariant>
}

// XXX: None
pub(crate) fn to_value<T>(x: &T) -> Result<Value, Error>
where
    T: Serialize
{
    let mut serializer = Serializer::default();
    let value = x.serialize(&mut serializer)?;
    let mut m = Map::new();
    m.insert("value".into(), value);
    m.insert(
        "handles".into(),
        serde_json::to_value(serializer.handles())?
    );
    Ok(m.into())
}

impl Serializer {
    fn handles(self) -> Vec<OnlyGuid> { self.handles.take() }
}

fn convert_kv<M: IntoIterator<Item = (String, Value)>>(map: M) -> Map<String, Value> {
    let entries = Value::Array(
        map.into_iter()
            .map(|(k, v)| {
                let mut entry = Map::new();
                entry.insert("k".into(), k.into());
                entry.insert("v".into(), v);
                Value::from(entry)
            })
            .collect()
    );
    let mut res = Map::new();
    res.insert("o".into(), entries);
    res
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = Value;
    type Error = Error;

    type SerializeSeq = &'a mut Seq;
    type SerializeTuple = &'a mut Seq;
    type SerializeTupleStruct = &'a mut Seq;
    type SerializeTupleVariant = &'a mut TupleVariant;
    type SerializeMap = &'a mut ObjectM;
    type SerializeStruct = &'a mut ObjectS;
    type SerializeStructVariant = &'a mut StructVariant;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        let mut m = Map::new();
        m.insert("b".into(), v.into());
        Ok(m.into())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        let mut m = Map::new();
        m.insert("n".into(), v.into());
        Ok(m.into())
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> { self.serialize_i64(v.into()) }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> { self.serialize_i64(v.into()) }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> { self.serialize_i64(v.into()) }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        let mut m = Map::new();
        m.insert("n".into(), v.into());
        Ok(m.into())
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> { self.serialize_u64(v.into()) }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> { self.serialize_u64(v.into()) }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> { self.serialize_u64(v.into()) }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        let mut m = Map::new();
        if v.is_nan() {
            m.insert("v".into(), "NaN".into())
        } else if v.is_infinite() {
            m.insert(
                "v".into(),
                if v.is_sign_negative() {
                    "-Infinity"
                } else {
                    "Infinity"
                }
                .into()
            )
        } else if v.is_sign_negative() && v == -0.0 {
            m.insert("v".into(), "-0".into())
        } else {
            m.insert("n".into(), v.into())
        };
        Ok(m.into())
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> { Ok(f64::from(v).into()) }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let mut m = Map::new();
        m.insert("s".into(), v.into());
        Ok(m.into())
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let mut m = Map::new();
        m.insert("s".into(), v.to_string().into());
        Ok(m.into())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Error::NotSupported)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        let mut m = Map::new();
        m.insert("v".into(), "null".into());
        Ok(m.into())
    }
    fn serialize_some<T>(self, v: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize
    {
        v.serialize(self)
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        let mut m = Map::new();
        m.insert("v".into(), "undefined".into());
        Ok(m.into())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize
    {
        let map = convert_kv(Some((variant.into(), value.serialize(self)?)));
        Ok(map.into())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.seq.push(Seq::new(self.clone()));
        Ok(self.seq.last_mut().unwrap())
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeSeq, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize
    ) -> Result<Self::SerializeSeq, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.t.push(TupleVariant::new(self.clone(), variant));
        Ok(self.t.last_mut().unwrap())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.om.push(ObjectM::new(self.clone()));
        Ok(self.om.last_mut().unwrap())
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.os.push(ObjectS::new(self.clone(), name));
        Ok(self.os.last_mut().unwrap())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.s.push(StructVariant::new(self.clone(), variant));
        Ok(self.s.last_mut().unwrap())
    }
}

#[derive(Clone)]
pub(crate) struct Seq {
    values: Vec<Value>,
    prime: Serializer
}

impl Seq {
    fn new(prime: Serializer) -> Self {
        Self {
            values: Vec::new(),
            prime
        }
    }
}

impl<'a> ser::SerializeSeq for &'a mut Seq {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize
    {
        self.values.push(value.serialize(&mut self.prime)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut m = Map::new();
        let mut vs = Vec::new();
        mem::swap(&mut self.values, &mut vs);
        m.insert("a".into(), vs.into());
        Ok(m.into())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Seq {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize
    {
        self.values.push(value.serialize(&mut self.prime)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut m = Map::new();
        let mut vs = Vec::new();
        mem::swap(&mut self.values, &mut vs);
        m.insert("a".into(), vs.into());
        Ok(m.into())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Seq {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize
    {
        self.values.push(value.serialize(&mut self.prime)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut m = Map::new();
        let mut vs = Vec::new();
        mem::swap(&mut self.values, &mut vs);
        m.insert("a".into(), vs.into());
        Ok(m.into())
    }
}

#[derive(Clone)]
pub(crate) struct TupleVariant {
    values: Vec<Value>,
    variant: &'static str,
    prime: Serializer
}

impl TupleVariant {
    fn new(prime: Serializer, variant: &'static str) -> Self {
        Self {
            values: Vec::new(),
            variant,
            prime
        }
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut TupleVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize
    {
        self.values.push(value.serialize(&mut self.prime)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let a = {
            let mut a = Map::new();
            let mut vs = Vec::new();
            mem::swap(&mut self.values, &mut vs);
            a.insert("a".into(), vs.into());
            a
        };
        let o = convert_kv(Some((self.variant.into(), a.into())));
        Ok(o.into())
    }
}

#[derive(Clone)]
pub(crate) struct ObjectS {
    name: &'static str,
    map: Map<String, Value>,
    prime: Serializer,
    guid: Option<Str<Guid>>,
    d: Option<String>
}

#[derive(Clone)]
pub(crate) struct ObjectM {
    keys: Vec<String>,
    values: Vec<Value>,
    prime: Serializer,
    turn: bool
}

impl ObjectS {
    fn new(prime: Serializer, name: &'static str) -> Self {
        Self {
            name,
            prime,
            map: Map::new(),
            guid: None,
            d: None
        }
    }
}

impl ObjectM {
    fn new(prime: Serializer) -> Self {
        Self {
            prime,
            keys: Vec::new(),
            values: Vec::new(),
            turn: false
        }
    }
}

impl<'a> ser::SerializeStruct for &'a mut ObjectS {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize
    {
        let v = value.serialize(&mut self.prime)?;
        if (self.name == "4a9c3811-6f00-49e5-8a81-939f932d9061"
            || self.name == "fff9ae7f-9070-480f-9a8a-3d4b66923f7d")
            && key == "guid"
        {
            let g = match v {
                Value::Object(m) => {
                    let (_, v) = m.into_iter().next().ok_or(Error::JsHandle)?;
                    let s = match v {
                        Value::String(s) => s,
                        _ => return Err(Error::JsHandle)
                    };
                    Str::validate(s).unwrap()
                }
                _ => return Err(Error::JsHandle)
            };
            self.guid = Some(g);
            return Ok(());
        }
        if self.name == "e7ee19d3-64cb-4286-8762-6dd8ab78eb89" && key == "d" {
            let d = match v {
                Value::Object(m) => {
                    let (_, v) = m.into_iter().next().ok_or(Error::DateTime)?;
                    match v {
                        Value::String(s) => s,
                        _ => return Err(Error::DateTime)
                    }
                }
                _ => return Err(Error::JsHandle)
            };
            self.d = Some(d);
            return Ok(());
        }
        self.map.insert(key.into(), v);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.name == "4a9c3811-6f00-49e5-8a81-939f932d9061"
            || self.name == "fff9ae7f-9070-480f-9a8a-3d4b66923f7d"
        {
            let handles = &mut self.prime.handles.borrow_mut();
            let guid = self.guid.take().ok_or(Error::JsHandle)?;
            handles.push(OnlyGuid { guid });
            let idx = handles.len() - 1;
            let mut m = Map::new();
            m.insert("h".into(), idx.into());
            Ok(m.into())
        } else if self.name == "e7ee19d3-64cb-4286-8762-6dd8ab78eb89" {
            let d = self.d.take().ok_or(Error::DateTime)?;
            let mut m = Map::new();
            m.insert("d".into(), d.into());
            Ok(m.into())
        } else {
            let mut m = Map::new();
            mem::swap(&mut self.map, &mut m);
            let o = convert_kv(m);
            Ok(o.into())
        }
    }
}

impl<'a> ser::SerializeMap for &'a mut ObjectM {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize
    {
        if self.turn {
            return Err(Error::OddMap);
        }
        let serialized = key.serialize(&mut self.prime)?;
        let key = match serialized {
            Value::String(s) => s,
            _ => return Err(Error::InvalidKey)
        };
        self.keys.push(key);
        self.turn = true;
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize
    {
        if !self.turn {
            return Err(Error::OddMap);
        }
        self.values.push(value.serialize(&mut self.prime)?);
        self.turn = false;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.turn {
            return Err(Error::OddMap);
        }
        let mut ks = Vec::new();
        let mut vs = Vec::new();
        mem::swap(&mut self.keys, &mut ks);
        mem::swap(&mut self.values, &mut vs);
        let entries = ks.into_iter().zip(vs.into_iter());
        let o = convert_kv(entries);
        Ok(o.into())
    }
}

#[derive(Clone)]
pub(crate) struct StructVariant {
    m: Map<String, Value>,
    variant: &'static str,
    prime: Serializer
}

impl StructVariant {
    fn new(prime: Serializer, variant: &'static str) -> Self {
        Self {
            m: Map::new(),
            variant,
            prime
        }
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut StructVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize
    {
        self.m.insert(key.into(), value.serialize(&mut self.prime)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut m = Map::new();
        mem::swap(&mut self.m, &mut m);
        let inner = convert_kv(m);
        let external = convert_kv(Some((self.variant.into(), inner.into())));
        Ok(external.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r#struct() {
        #[derive(Serialize)]
        struct Test {
            int: u32,
            seq: Vec<&'static str>
        }

        let test = Test {
            int: 1,
            seq: vec!["a", "b"]
        };
        let expected = r#"{
                "value":{"o":[{"k":"int","v":{"n":1}},{"k":"seq","v":{"a": [{"s":"a"},{"s":"b"}]}}]},
                "handles":[]}"#;
        let v: Value = serde_json::from_str(expected).unwrap();
        dbg!(&v);
        assert_eq!(to_value(&test).unwrap(), v);
    }

    #[test]
    fn option() {
        let expected = r#"{
            "value":{"n":3},
            "handles": []}"#;
        let v: Value = serde_json::from_str(expected).unwrap();
        assert_eq!(to_value(&Some(3)).unwrap(), v);
        let expected = r#"{
            "value":{"v":"null"},
            "handles": []}"#;
        let v: Value = serde_json::from_str(expected).unwrap();
        assert_eq!(to_value(&Option::<i32>::None).unwrap(), v);
    }

    #[test]
    fn r#enum() {
        #[derive(Serialize)]
        enum E {
            Unit,
            Newtype(u32),
            Tuple(u32, u32),
            Struct { a: u32 }
        }

        let u = E::Unit;
        let expected = r#"{"value":{"s":"Unit"}, "handles":[]}"#;
        let v: Value = serde_json::from_str(expected).unwrap();
        assert_eq!(to_value(&u).unwrap(), v);

        let u = E::Newtype(1);
        let expected = r#"{"value":{"o":[{"k":"Newtype","v":{"n":1}}]}, "handles":[]}"#;
        let v: Value = serde_json::from_str(expected).unwrap();
        assert_eq!(to_value(&u).unwrap(), v);

        let u = E::Tuple(1, 2);
        let expected =
            r#"{"value": {"o":[{"k":"Tuple","v":{"a":[{"n":1},{"n":2}]}}]}, "handles":[]}"#;
        let v: Value = serde_json::from_str(expected).unwrap();
        assert_eq!(to_value(&u).unwrap(), v);

        let u = E::Struct { a: 1 };
        let expected =
            r#"{"value":{"o":[{"k":"Struct","v":{"o":[{"k":"a","v":{"n":1}}]}}]},"handles":[]}"#;
        let v: Value = serde_json::from_str(expected).unwrap();
        assert_eq!(to_value(&u).unwrap(), v);
    }
}
