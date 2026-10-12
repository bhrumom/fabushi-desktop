use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use mahayana_host_runtime::extensions::attachments::link_preview_image_bounds::*;

fn png(w:u32,h:u32)->Vec<u8>{let mut b=vec![0;24];b[..8].copy_from_slice(&[137,80,78,71,13,10,26,10]);b[12..16].copy_from_slice(b"IHDR");b[16..20].copy_from_slice(&w.to_be_bytes());b[20..24].copy_from_slice(&h.to_be_bytes());b}
fn url(b:&[u8])->String{format!("data:image/png;base64,{}",STANDARD.encode(b))}

#[test]
fn binary_dimension_readers_cover_frozen_formats(){
 assert_eq!(read_png_dimensions(&png(640,480)),Some(ImageSize{width:640,height:480}));
 let mut gif=b"GIF89a".to_vec();gif.extend_from_slice(&320u16.to_le_bytes());gif.extend_from_slice(&200u16.to_le_bytes());
 assert_eq!(read_gif_dimensions(&gif),Some(ImageSize{width:320,height:200}));
 let mut webp=vec![0;30];webp[0..4].copy_from_slice(b"RIFF");webp[8..12].copy_from_slice(b"WEBP");webp[12..16].copy_from_slice(b"VP8X");
 let w=499u32;let h=299u32;webp[24]=(w&255)as u8;webp[25]=((w>>8)&255)as u8;webp[26]=((w>>16)&255)as u8;webp[27]=(h&255)as u8;webp[28]=((h>>8)&255)as u8;webp[29]=((h>>16)&255)as u8;
 assert_eq!(read_webp_dimensions(&webp),Some(ImageSize{width:500,height:300}));
 let mut ico=vec![0;34];ico[2..4].copy_from_slice(&1u16.to_le_bytes());ico[4..6].copy_from_slice(&1u16.to_le_bytes());ico[18..22].copy_from_slice(&22u32.to_le_bytes());ico[22..26].copy_from_slice(&40u32.to_le_bytes());ico[26..30].copy_from_slice(&512i32.to_le_bytes());ico[30..34].copy_from_slice(&1024i32.to_le_bytes());
 assert_eq!(read_ico_size(&ico),Some(ImageSize{width:512,height:512}));
}
#[test]
fn data_url_pixel_guard_and_resize_fallback_match_frozen_behavior(){
 let original=url(&png(400,200));assert_eq!(read_encoded_image_size(&original),Some(ImageSize{width:400,height:200}));
 let bounds=PreviewImageBounds{max_dimension:100,encoding:"webp"};
 let resized=bound_preview_image_data_url(Some(&original),&bounds,|_,target,encoding|{assert_eq!(target,ResizeTarget::Width(100));assert_eq!(*encoding,"webp");Some("resized".into())});
 assert_eq!(resized.as_deref(),Some("resized"));
 assert_eq!(bound_preview_image_data_url(Some(&original),&bounds,|_,_,_|None).as_deref(),Some(original.as_str()));
 let huge=url(&png(6000,5000));assert_eq!(bound_preview_image_data_url(Some(&huge),&bounds,|_,_,_|Some("bad".into())),None);
}

fn iso_box(kind:&[u8;4],body:&[u8])->Vec<u8>{
 let size=u32::try_from(body.len()+8).expect("box size");
 let mut out=Vec::with_capacity(body.len()+8);
 out.extend_from_slice(&size.to_be_bytes());out.extend_from_slice(kind);out.extend_from_slice(body);out
}
fn full_box(kind:&[u8;4],payload:&[u8])->Vec<u8>{
 let mut body=vec![0,0,0,0];body.extend_from_slice(payload);iso_box(kind,&body)
}

#[test]
fn bmff_clean_aperture_is_applied_before_rotation(){
 let mut ftyp_body=b"avif".to_vec();ftyp_body.extend_from_slice(&0u32.to_be_bytes());
 let mut bytes=iso_box(b"ftyp",&ftyp_body);
 let mut ispe_payload=Vec::new();ispe_payload.extend_from_slice(&800u32.to_be_bytes());ispe_payload.extend_from_slice(&600u32.to_be_bytes());
 let ispe=full_box(b"ispe",&ispe_payload);
 let mut clap_payload=Vec::new();
 for value in [400u32,1,300,1,0,1,0,1] {clap_payload.extend_from_slice(&value.to_be_bytes());}
 let clap=iso_box(b"clap",&clap_payload);let irot=iso_box(b"irot",&[1]);
 let mut ipco_body=Vec::new();ipco_body.extend_from_slice(&ispe);ipco_body.extend_from_slice(&clap);ipco_body.extend_from_slice(&irot);
 let ipco=iso_box(b"ipco",&ipco_body);
 let mut ipma_body=vec![0,0,0,0];ipma_body.extend_from_slice(&1u32.to_be_bytes());ipma_body.extend_from_slice(&1u16.to_be_bytes());ipma_body.push(3);ipma_body.extend_from_slice(&[1,2,3]);
 let ipma=iso_box(b"ipma",&ipma_body);
 let mut iprp_body=Vec::new();iprp_body.extend_from_slice(&ipco);iprp_body.extend_from_slice(&ipma);
 let iprp=iso_box(b"iprp",&iprp_body);let pitm=full_box(b"pitm",&1u16.to_be_bytes());
 let mut meta_body=vec![0,0,0,0];meta_body.extend_from_slice(&pitm);meta_body.extend_from_slice(&iprp);
 bytes.extend_from_slice(&iso_box(b"meta",&meta_body));
 assert_eq!(read_heic_dimensions(&bytes),Some(ImageSize{width:300,height:400}));
}
