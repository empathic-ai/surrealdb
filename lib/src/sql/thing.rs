use crate::ctx::Context;
use crate::dbs::{Options, Transaction};
use crate::doc::CursorDoc;
use crate::err::Error;
use crate::sql::{escape::escape_rid, id::Id, Strand, Value};
use crate::syn;
use derive::Store;
use prost::DecodeError;
use revision::revisioned;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use bevy::prelude::Reflect;
use prost::encoding::WireType;
use prost::encoding::DecodeContext;

pub(crate) const TOKEN: &str = "$surrealdb::private::sql::Thing";

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Store, Hash, Reflect)]
#[serde(rename = "$surrealdb::private::sql::Thing")]
#[revisioned(revision = 1)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Thing {
	/// Table name
	pub tb: String,
	pub id: Id,
}

impl Default for Thing {
    fn default() -> Self {
        Self { tb: Default::default(), id: Default::default() }
    }
}

// See for guidance:
// https://github.com/tokio-rs/prost/issues/882
impl ::prost::Message for Thing {
    fn encode_raw<B>(&self, buf: &mut B)
    where
        B: bytes::BufMut,
        Self: Sized {
		let str = format!("{}:{}", self.tb, self.id.to_string());
		prost::encoding::string::encode(1u32, &str, buf);
    }

    fn merge_field<B>(
        &mut self,
        tag: u32,
        wire_type: WireType,
        buf: &mut B,
        ctx: DecodeContext,
    ) -> Result<(), prost::DecodeError>
    where
        B: bytes::Buf,
        Self: Sized {

		let err_ctx = |mut error: DecodeError| {
			error.push("Foo", "int_or_string");
			error
		};

		match tag {
			1u32 => {
				match wire_type {
					WireType::LengthDelimited => {
						let mut value = String::new();
						prost::encoding::string::merge(wire_type, &mut value, buf, ctx)
							.map_err(err_ctx)?;

						let values: Vec<&str> = value.split(':').collect();

						self.tb = values[0].to_string();
						self.id = Id::String(values[1].to_string());
						Ok(())
					},
					_ =>
						Err(err_ctx(DecodeError::new(format!(
							"invalid wire type: {:?} (expected {:?} or {:?})",
							wire_type, WireType::Varint, WireType::LengthDelimited
						)))
					)
				}
			},
			_ => prost::encoding::skip_field(wire_type, tag, buf, ctx),
		}
    }

    fn encoded_len(&self) -> usize {
		let str = format!("{}:{}", self.tb, self.id.to_string());
		prost::encoding::string::encoded_len(1u32, &str)
    }

    fn clear(&mut self) {
		self.tb = Default::default();
        self.id = Default::default();
    }
}

impl From<(&str, Id)> for Thing {
	fn from((tb, id): (&str, Id)) -> Self {
		Self {
			tb: tb.to_owned(),
			id,
		}
	}
}

impl From<(String, Id)> for Thing {
	fn from((tb, id): (String, Id)) -> Self {
		Self {
			tb,
			id,
		}
	}
}

impl From<(String, String)> for Thing {
	fn from((tb, id): (String, String)) -> Self {
		Self::from((tb, Id::from(id)))
	}
}

impl From<(&str, &str)> for Thing {
	fn from((tb, id): (&str, &str)) -> Self {
		Self::from((tb.to_owned(), Id::from(id)))
	}
}

impl FromStr for Thing {
	type Err = ();
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::try_from(s)
	}
}

impl TryFrom<String> for Thing {
	type Error = ();
	fn try_from(v: String) -> Result<Self, Self::Error> {
		Self::try_from(v.as_str())
	}
}

impl TryFrom<Strand> for Thing {
	type Error = ();
	fn try_from(v: Strand) -> Result<Self, Self::Error> {
		Self::try_from(v.as_str())
	}
}

impl TryFrom<&str> for Thing {
	type Error = ();
	fn try_from(v: &str) -> Result<Self, Self::Error> {
		match syn::thing(v) {
			Ok(v) => Ok(v),
			_ => Err(()),
		}
	}
}

impl Thing {
	/// Convert the Thing to a raw String
	pub fn to_raw(&self) -> String {
		self.to_string()
	}
}

impl fmt::Display for Thing {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}:{}", escape_rid(&self.tb), self.id)
	}
}

impl Thing {
	/// Process this type returning a computed simple Value
	pub(crate) async fn compute(
		&self,
		ctx: &Context<'_>,
		opt: &Options,
		txn: &Transaction,
		doc: Option<&CursorDoc<'_>>,
	) -> Result<Value, Error> {
		Ok(Value::Thing(Thing {
			tb: self.tb.clone(),
			id: self.id.compute(ctx, opt, txn, doc).await?,
		}))
	}
}
