use seq::seq;

seq!(N in 0..=1 {
    #[repr(u8)]
    enum Test0 {
        #(Variant~N,)*
    }

    #[repr(u8)]
    enum Test1 {
        #(Variant~N,)*
    }
});

seq!(N in 0..=1 {
    mod test~N {
        #(
            #[repr(u8)]
            pub enum Test~N {
                #(Variant~N,)*
            }
        )*
    }
});

fn main() {
    let _ = Test0::Variant0;
    let _ = Test0::Variant1;

    let _ = Test1::Variant0;
    let _ = Test1::Variant1;

    let _ = test0::Test0::Variant0;
    let _ = test0::Test0::Variant1;

    let _ = test0::Test1::Variant0;
    let _ = test0::Test1::Variant1;

    let _ = test1::Test0::Variant0;
    let _ = test1::Test0::Variant1;

    let _ = test1::Test1::Variant0;
    let _ = test1::Test1::Variant1;
}
