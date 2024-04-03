use alohomora::pcr::{PrivacyCriticalRegion, Signature};

fn main(){
    //TODO passing just plus_two doesn't even trigger the lint bc its passing as not a closure
    let plus_two = |x: u8| { println!("{}", x) };  
    
    let _pcr = PrivacyCriticalRegion::new( |x: u8 | { plus_two(x) }, 
    Signature {username: "corinnt", 
                signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUFlNmczSkFuQWltekJTNGRWdzZLdUFoWGVCWloyQnN4M045NVFCZjM2MWpRTEpMYWRIeFRvRWhzMEVpNHNvZk4KWUJudUlHdmF2WGU2cG9kVWJ5U3hRQwotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"}, 
  Signature {username: "corinnt", 
                signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUFlNmczSkFuQWltekJTNGRWdzZLdUFoWGVCWloyQnN4M045NVFCZjM2MWpRTEpMYWRIeFRvRWhzMEVpNHNvZk4KWUJudUlHdmF2WGU2cG9kVWJ5U3hRQwotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"}); 
}
