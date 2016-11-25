use uuid::Uuid;
use std::io::Write;
use std::i32;
use std::str;
use std::u8;
use std::io::Read;
use std::io::{Cursor, Error as IoError};
use models;
use chrono::NaiveDateTime;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub enum KeyComponent {
	Uuid(Uuid),
	UnsizedString(String),
	ShortSizedString(String),
	NaiveDateTime(NaiveDateTime)
}

impl KeyComponent {
	fn len(&self) -> usize {
		match *self {
			KeyComponent::Uuid(_) => 16,
			KeyComponent::UnsizedString(ref s) | KeyComponent::ShortSizedString(ref s) => s.len(),
			KeyComponent::NaiveDateTime(_) => 8
		}
	}

	fn write(&self, cursor: &mut Cursor<Vec<u8>>) -> Result<(), IoError> {
		match *self {
			KeyComponent::Uuid(ref uuid) => {
				try!(cursor.write(uuid.as_bytes()));
			},
			KeyComponent::UnsizedString(ref s) => {
				try!(cursor.write(s.as_bytes()));
			},
			KeyComponent::ShortSizedString(ref s) => {
				debug_assert!(s.len() <= u8::MAX as usize);
				try!(cursor.write(&[s.len() as u8]));
				try!(cursor.write(s.as_bytes()));
			},
			KeyComponent::NaiveDateTime(ref datetime) => {
				let timestamp = datetime.timestamp();
				debug_assert!(timestamp >= 0);
				try!(cursor.write_i64::<BigEndian>(timestamp));
			}
		};

		Ok(())
	}
}

pub fn build_key(components: Vec<KeyComponent>) -> Box<[u8]> {
	let len = components.iter().fold(0, |len, ref component| len + component.len());
	let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::with_capacity(len));

	for component in components.iter() {
		if let Err(err) = component.write(&mut cursor) {
			panic!("Could not build key: {}", err);
		}
	}

	cursor.into_inner().into_boxed_slice()
}

pub fn parse_uuid_key(key: Box<[u8]>) -> Uuid {
	debug_assert_eq!(key.len(), 16);
	let mut cursor = Cursor::new(key);
	read_uuid(&mut cursor)
}

pub fn read_uuid(cursor: &mut Cursor<Box<[u8]>>) -> Uuid {
	let mut buf: [u8; 16] = [0; 16];
	cursor.read_exact(&mut buf).unwrap();
	Uuid::from_bytes(&buf).unwrap()
}

pub fn read_short_sized_string(cursor: &mut Cursor<Box<[u8]>>) -> String {
	let t_len = {
		let mut buf: [u8; 1] = [0; 1];
		cursor.read_exact(&mut buf).unwrap();
		buf[0] as usize
	};
	
	let mut buf = vec![0u8; t_len];
	cursor.read_exact(&mut buf).unwrap();
	str::from_utf8(&buf).unwrap().to_string()
}

pub fn read_type(mut cursor: &mut Cursor<Box<[u8]>>) -> models::Type {
	models::Type::new(read_short_sized_string(&mut cursor)).unwrap()
}

pub fn read_unsized_string(cursor: &mut Cursor<Box<[u8]>>) -> String {
	let mut buf = String::new();
	cursor.read_to_string(&mut buf).unwrap();
	buf

}

pub fn read_datetime(cursor: &mut Cursor<Box<[u8]>>) -> NaiveDateTime {
	let timestamp = cursor.read_i64::<BigEndian>().unwrap();
    NaiveDateTime::from_timestamp(timestamp, 0)
}

pub fn max_uuid() -> Uuid {
	Uuid::from_bytes(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]).unwrap()
}

pub fn max_datetime() -> NaiveDateTime {
	// NOTE: this suffers from the year 2038 problem, but we can't use
	// i64::MAX because chrono sees it as an invalid time
	NaiveDateTime::from_timestamp(i32::MAX as i64, 0)
}
