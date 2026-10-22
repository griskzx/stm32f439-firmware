use defmt::info;
use stm32f4xx_hal::crc32::Crc32;

const CMD_LEN:usize = max_len();
const DOWNLOAD:&[u8] = b"download";
const JUMP:&[u8] = b"jump";
const UPDATE:&[u8] = b"update";
const APP_MSG_FRAME_HEAD:&str = "Product:unkown/Authors:xin/Type:app message";
const APP_MSG_FRAME_HEAD_LEN:usize = APP_MSG_FRAME_HEAD.len();

const APP_MSG_FRAME_TAIL:&str = "/end";
const APP_MSG_FRAME_TAIL_LEN:usize = APP_MSG_FRAME_TAIL.len();

const APP_FIRMWARE_FRAME_HEAD:&str = "Product:unkown/Authors:xin/Type:app firmware";
const APP_FIRMWARE_FRAME_HEAD_LEN:usize = APP_FIRMWARE_FRAME_HEAD.len();

use APP_MSG_FRAME_TAIL as  APP_FRAME_TAIL;

use APP_MSG_FRAME_TAIL_LEN as APP_FRAME_TAIL_LEN;

#[derive(Debug,Copy, Clone)]
pub enum DataType{
	Msg,
	Firmware
}

#[derive(Debug,Copy, Clone)]
pub enum AppComponent<'a>{
	Msg(crate::app::AppMsg),
	Firmware(&'a [u8])
}

impl From<DataType> for &str {
	fn from(value: DataType) -> Self {
		match value {
			DataType::Msg => APP_MSG_FRAME_HEAD,
			DataType::Firmware => APP_FIRMWARE_FRAME_HEAD,
		}
	}
}

impl<'a> From<DataType> for AppComponent<'a> {
	fn from(value: DataType) -> Self {
		match value {
			DataType::Msg => AppComponent::Msg(crate::app::AppMsg{
				version:[0;4],
				size:0,
				sha:[0;32],
				crc32:0
			}),
			DataType::Firmware => AppComponent::Firmware(&[]),
		}
	}
}


const fn max_len()->usize{
	let mut max = if DOWNLOAD.len()>JUMP.len() {  DOWNLOAD.len() } else { JUMP.len() };
	max =if UPDATE.len()>max { UPDATE.len() } else { max };
	max
}

pub enum Cmd{
	Download,
	Jump,
	Update,
	Unknown
}

pub struct Cli;
impl Cli{

	#[inline]
	pub fn cmd_parse(&self,s:&[u8])->Cmd{
		cmd_parse(s)
	}
	#[inline]
	pub fn msg_frame_parse(&self,s:&[u8])->Result<crate::app::AppMsg,()>{
		use zerocopy::FromBytes;
		let msg = frame_parse(s,DataType::Msg)?;
		let msg = crate::app::AppMsg::ref_from_bytes(msg).map_err(|_|())?;
		Ok(*msg)
	}
	pub fn firmware_frame_parse<'a>(&self,s:&'a [u8])->Result<&'a [u8],()>{
		frame_parse(s,DataType::Firmware)
	}

	pub fn check_crc32(&self,checker:&mut Crc32,data:&[u8])->bool{
		if data.len()<4 { return false }
		checker.init();
		let (data,crc)=data.split_at(data.len()-4);
		let rst_crc =checker.update_bytes(data).to_le_bytes();
		rst_crc == crc
		//unimplemented!()
	}
}

fn frame_parse(s: &[u8], data_type: DataType) ->Result<&[u8], ()> {
	let head:&str = data_type.into();
	let head_len = head.len();
	if s.len() < head_len+APP_FRAME_TAIL_LEN+crate::app::APP_MSG_SIZE { return Err(()); }
	if &s[..head_len] == APP_MSG_FRAME_HEAD.as_bytes() &&
		&s[s.len()-APP_FRAME_TAIL_LEN..] == APP_FRAME_TAIL.as_bytes() {
		let app = &s[head_len..s.len() - APP_FRAME_TAIL_LEN];
		if app.len() != crate::app::APP_MSG_SIZE { return Err(()) }
		Ok(app)
 	}else {
		Err(())
	}
}

fn cmd_parse(s:&[u8])->Cmd{
	match s {
		DOWNLOAD=>{
			info!("download ota package");
			Cmd::Download
		}
		JUMP =>{
			info!("jump to app");
			Cmd::Jump
		}
		UPDATE=>{
			info!("download and jump to app");
			Cmd::Update
		}
		_=>{
			info!("unknown command");
			Cmd::Unknown
		}
	}
}

#[cfg(test)]
mod tests{
	use super::*;
	#[test]
	fn channel(){
		let mut queue:heapless::spsc::Queue<heapless::vec::Vec<u8,CMD_LEN>,8> = heapless::spsc::Queue::new();
		let (mut rx,mut tx) = queue.split();
		let mut one:heapless::Vec<u8,CMD_LEN> = heapless::Vec::new();
		one.extend_from_slice(DOWNLOAD).unwrap();
		rx.enqueue(one).unwrap();
		let rst = tx.dequeue().unwrap();
		assert_eq!(rst.as_slice(),DOWNLOAD);
	}
}