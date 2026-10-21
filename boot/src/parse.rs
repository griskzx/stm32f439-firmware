use defmt::info;
const CMD_LEN:usize = max_len();
const DOWNLOAD:&[u8] = b"download";
const JUMP:&[u8] = b"jump";
const UPDATE:&[u8] = b"update";
const APP_MSG_FRAME_HEAD:&str = "Product:unkown/Authors:xin/Type:app message";
const APP_MSG_FRAME_HEAD_LEN:usize = APP_MSG_FRAME_HEAD.len();

const APP_MSG_FRAME_TAIL:&str = "/end";
const APP_MSG_FRAME_TAIL_LEN:usize = APP_MSG_FRAME_TAIL.len();

const fn max_len()->usize{
	let mut max = if DOWNLOAD.len()>JUMP.len() {  DOWNLOAD.len() } else { JUMP.len() };
	max =if UPDATE.len()>max { UPDATE.len() } else { max };
	max
}

pub struct Cli;
impl Cli{

	#[inline]
	pub fn cmd_parse(&self,s:&[u8]){
		cmd_parse(s)
	}
}

fn msg_frame_parse(s:&[u8])->Result<&crate::app::App,()>{
	if s.len() < APP_MSG_FRAME_HEAD_LEN+APP_MSG_FRAME_TAIL_LEN+crate::app::APP_MSG_SIZE { return Err(()); }
	if &s[..APP_MSG_FRAME_HEAD_LEN] == APP_MSG_FRAME_HEAD.as_bytes() &&
		&s[s.len()-APP_MSG_FRAME_TAIL_LEN..] == APP_MSG_FRAME_TAIL.as_bytes() {
		let app = &s[APP_MSG_FRAME_HEAD_LEN..s.len()-APP_MSG_FRAME_TAIL_LEN];
		
	}
	unimplemented!()
}

fn cmd_parse(s:&[u8]){
	match s {
		DOWNLOAD=>{
			info!("download ota package");
		}
		JUMP =>{
			info!("jump to app")
		}
		UPDATE=>{
			info!("download and jump to app");
		}
		_=>{
			info!("unknown command");
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