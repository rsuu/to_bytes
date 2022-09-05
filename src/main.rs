use std::{
    fs,
    fs::OpenOptions,
    io,
    io::{Read, Write},
    mem,
};

#[derive(Debug, Copy, Clone)]
#[repr(C)]
#[repr(packed)]
struct A<'a> {
    a: [u8; 2],
    s: &'a str,
}

#[derive(Debug, Clone)]
#[repr(C)]
struct B<'a> {
    a: [u8; 3],
    c: u8,
    d: Vec<u8>,
    s: &'a str,
    ss: String,

    aa: A<'a>,
}

impl<'a> TBytesExt for A<'a> {}
impl<'a> TBytesExt for B<'a> {}

// SHOULD panic
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct C<T> {
    a: T,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct Zero {}

impl<T: TBytesExt> TBytesExt for C<T> {}
impl TBytesExt for Zero {}

// EOF SHOULD panic

fn main() {
    test_a(); // OK
    test_b(); // OK

    test_c(); // SHOULD panic
}

trait TT {}
impl TT for u32 {}
impl TT for u64 {}

fn dyn_res() -> Box<dyn TT> {
    if true {
        Box::new(0_u32)
    } else {
        Box::new(0_u64)
    }
}

fn test_c() {
    let s: Vec<C<Zero>> = vec![C { a: Zero {} }, C { a: Zero {} }];

    let type_size: usize = mem::size_of::<C<Zero>>();
    // rustc -Z print-type-sizes 3.rs

    let mut bytes: Vec<u8> = Vec::with_capacity(type_size * s.len());

    for f in s.iter() {
        bytes.extend_from_slice(f.as_bytes());
        //println!("{:?}", bytes);
    }

    // write
    let file = "test";
    if let Ok(_) = fs::File::create(file) {
        append_bytes(file, bytes.as_slice()).unwrap();
    }

    let bytes = read_bytes("test").expect("");
    let bytes = bytes.as_slice();
    //println!("{:?}", bytes);

    // read
    let bytes_slice = ByteSlice::new::<C<Zero>>(bytes);
    assert_eq!(bytes_slice, None);
}

fn test_a() {
    let s: Vec<A> = vec![
        A {
            a: [0, 2],
            s: "cac",
        },
        A {
            a: [0, 1],
            s: "akasssssssssssssssssasdasdki",
        },
    ];

    let type_size: usize = mem::size_of::<A>();
    // rustc -Z print-type-sizes 3.rs

    let mut bytes: Vec<u8> = Vec::with_capacity(type_size * s.len());

    for f in s.iter() {
        bytes.extend_from_slice(f.as_bytes());
        //println!("{:?}", bytes);
    }

    // write
    let file = "test";
    if let Ok(_) = fs::File::create(file) {
        append_bytes(file, bytes.as_slice()).unwrap();
    }

    let bytes = read_bytes("test").expect("");
    let bytes = bytes.as_slice();
    //println!("{:?}", bytes);

    // read
    let mut bytes_slice = ByteSlice::new::<A>(bytes).unwrap();
    //println!("{}", bytes_slice.slice_len);
    let v = bytes_slice.as_vec_struct::<A>().expect("");
    println!("{:#?}", v);
}

fn test_b() {
    let s: Vec<B> = vec![
        B {
            a: [0, 2, 0],
            s: "cac",
            d: vec![1],
            c: 0,
            ss: "akasssssssssssssssssasdasdki".to_string(),
            aa: A {
                a: [0, 1],
                s: "akasssssssssssssssssasdasdki",
            },
        },
        B {
            a: [0, 1, 0],
            d: vec![1],
            c: 1,
            s: "akasssssssssssssssssasdasdki",
            ss: "akasssssssssssssssssasdasdki".to_string(),
            aa: A {
                a: [0, 1],
                s: "akasssssssssssssssssasdasdki",
            },
        },
    ];

    let bytes = ByteSlice::to_bytes(&s);

    let file = "test";
    if let Ok(_) = fs::File::create(file) {
        append_bytes(file, bytes.as_slice()).unwrap();
    }

    let bytes = read_bytes("test").expect("");

    let type_size: usize = mem::size_of::<B>();
    let mut bytes_slice = ByteSlice::new::<B>(bytes.as_slice()).unwrap();
    let v = bytes_slice.into_vec_struct::<B>().expect("");
    println!("{:#?}", v);
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
struct ByteSlice<'a> {
    slice: &'a [u8],
    size: usize,
    slice_len: usize,
    max: usize,
    start: usize,
}

impl<'a> ByteSlice<'a> {
    pub fn new<T>(slice: &'a [u8]) -> Option<Self>
    where
        T: Sized + TBytesExt,
    {
        let size = std::mem::size_of::<T>();
        if size == 0 {
            None
        } else {
            Some(Self {
                slice,
                size,
                slice_len: slice.len(),
                max: slice.len() / size,
                start: 0,
            })
        }
    }

    pub fn read(&mut self) -> &[u8] {
        //println!("{}", self.slice_len);

        let s = &(self.slice[self.start..self.start + self.size]);
        self.to_next();

        //println!("{:?}", s);

        s
    }

    pub fn to_next(&mut self) {
        (*self).start += self.size;
    }

    pub fn as_struct<T>(&mut self) -> &T
    where
        T: Sized + Copy + TBytesExt,
    {
        <T as TBytesExt>::from_ref(self.read())
    }

    pub fn as_vec_struct<T>(&mut self) -> Result<Vec<T>, ()>
    where
        T: Sized + Copy + TBytesExt,
    {
        if self.max > 0 {
            let mut v: Vec<T> = Vec::with_capacity(self.slice_len);

            for _f in 0..self.max {
                // println!("{}", _f);

                v.push(*(self.as_struct::<T>()));
            }

            Ok(v)
        } else {
            Err(())
        }
    }

    pub fn to_struct<T>(&mut self) -> &T
    where
        T: Sized + Clone + TBytesExt,
    {
        <T as TBytesExt>::from_ref(self.read())
    }

    pub fn into_vec_struct<T>(&mut self) -> Result<Vec<T>, ()>
    where
        T: Sized + Clone + TBytesExt,
    {
        if self.max > 0 {
            let mut v: Vec<T> = Vec::with_capacity(self.max);

            for _ in 0..self.max {
                // println!("{}", _f);

                v.push(self.to_struct::<T>().clone());
            }

            Ok(v)
        } else {
            Err(())
        }
    }

    pub fn to_bytes<T>(slice: &[T]) -> Vec<u8>
    where
        T: Sized + TBytesExt,
    {
        let type_size: usize = mem::size_of::<T>();
        let mut bytes: Vec<u8> = Vec::with_capacity(type_size * slice.len());

        for val in slice.iter() {
            bytes.extend_from_slice(val.as_bytes());
        }

        bytes
    }
}

trait TBytesExt
where
    Self: Sized,
{
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }

    fn from_ref(buf: &[u8]) -> &Self {
        let p: *const Self = buf.as_ptr() as *const Self;

        unsafe { &*p }

        // transmute from slice [u8] to A: 23.358µs
        // transmute_copy from slice [u8] to A: 21.771µs
        // convert from slice [u8] to &A: 1.426µs
    }
}

pub fn read_bytes(file: &str) -> io::Result<Vec<u8>> {
    let mut f = fs::File::open(&file)?;
    let metadata = fs::metadata(&file)?;
    let mut buffer = vec![0; metadata.len() as usize];

    f.read_exact(&mut buffer)?;

    Ok(buffer)
}

pub fn append_bytes(file: &str, bytes: &[u8]) -> io::Result<()> {
    let mut f = OpenOptions::new().write(true).append(true).open(file)?;

    f.write_all(bytes)
}

// REFS:
// https://github.com/rust-lang/project-safe-transmute/blob/master/rfcs/0000-ext-byte-transmutation.md
// https://www.reddit.com/r/rust/comments/dw2vb3/convert_from_u8_to_generic_sized_struct/
