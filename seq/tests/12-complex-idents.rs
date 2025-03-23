use seq::seq;

seq!(N in 2..=2 {
    fn test~N() -> u32 {
        N
    }

    fn test~N~test() -> u32 {
        N
    }

    fn test~N~test~N~test() -> u32 {
        N
    }

    fn test~N~~N~~N() -> u32 {
        N
    }

    fn test~N~45~N() -> u32 {
        N
    }

    fn test~N~0xab~N() -> u32 {
        N
    }

    fn test~N~0o123~N() -> u32 {
        N
    }

    fn test~N~0b1010~N() -> u32 {
        N
    }

    fn test~N~0u32~N() -> u32 {
        N
    }

    fn test~N~45f32~N() -> u32 {
        N
    }
});

fn main() {
    let test = test2()
        + test2test()
        + test2test2test()
        + test222()
        + test2452()
        + test20xab2()
        + test20o1232()
        + test20b10102()
        + test20u322()
        + test245f322();

    assert_eq!(test, 20);
}
