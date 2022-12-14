//! File and filesystem-related syscalls
use crate::fs::count_nlink;
use crate::mm::translated_byte_buffer;
use crate::mm::translated_str;
use crate::mm::translated_refmut;
use crate::task::current_user_token;
use crate::task::current_task;
use crate::fs::{open_file,link_file,unlink_file};
use crate::fs::OpenFlags;
use crate::fs::Stat;
use crate::mm::UserBuffer;
use alloc::sync::Arc;
use alloc::task;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.read(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(
        path.as_str(),
        OpenFlags::from_bits(flags).unwrap()
    ) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

// YOUR JOB: ?????? easy-fs ?????????????????????????????? syscall
pub fn sys_fstat(_fd: usize, _st: *mut Stat) -> isize {
    //println!("begin sys fstat");
    let token=current_user_token();
    let st=translated_refmut(token, _st);
    let task=current_task().unwrap();
    let inner=task.inner_exclusive_access();
    //println!("begin get info");
    if _fd>2&&_fd<inner.fd_table.len(){
        st.dev=0;
        let ino=inner.get_ino(_fd);
        //println!("end get ino");
        st.ino=ino as u64;
        st.mode=inner.get_file_type(_fd);
        //println!("end get file mode");
        st.nlink=count_nlink(ino);
        //println!("end count nlink");
        0
    }else{
        -1
    }
}

pub fn sys_linkat(_old_name: *const u8, _new_name: *const u8) -> isize {
    let token=current_user_token();
    let old_name=translated_str(token, _old_name);
    let new_name=translated_str(token, _new_name);
    link_file(old_name.as_str(),new_name.as_str())
}

pub fn sys_unlinkat(_name: *const u8) -> isize {
    let token=current_user_token();
    let name=translated_str(token, _name);
    unlink_file(name.as_str())
}
