extern crate image;
extern crate rand;
use image::{ImageBuffer, Rgb, RgbImage, DynamicImage, GrayImage, Luma};
use rand::random;

struct TextureHash{
    blockcounts: [u32;65536],
    total: u32
}

fn readhash_to(img: &impl BinaryPixelAccess, hash: & mut TextureHash){
    let (w,h) = img.get_dimensions();
    hash.total += (w-3)*(h-3);
    for y in 0..h-3{
        for x in 0..w-3{
            hash.blockcounts[readblock(img, x,y) as usize] += 1;
        }
    }
}
    
fn readhash(img: &impl BinaryPixelAccess) -> TextureHash{
    let mut hash = TextureHash{
        blockcounts : [0;65536],
        total: 0
    };
    readhash_to(img, &mut hash);
    return hash;
}

trait BinaryPixelAccess{
    fn get_dimensions(&self)->(u32,u32);
    fn get_pixel(&self, x: u32, y:u32)->bool;
}

impl BinaryPixelAccess for GrayImage{
    fn get_dimensions(&self)->(u32,u32){ self.dimensions() }
    fn get_pixel(&self, x:u32, y:u32)->bool{
        self.get_pixel(x,y)[0] > 128
    }
}
impl BinaryPixelAccess for BitImage{
    fn get_dimensions(&self)->(u32,u32){ (self.width, self.height) }
    fn get_pixel(&self, x:u32, y:u32)->bool{self.get_pixel(x,y)!=0}
}

fn readblock( img: &impl BinaryPixelAccess, x: u32, y:u32 ) -> u32{
    let mut code: u32 = 0;
    let mut bit: u32 = 1;
    for xx in x..x+4{
        for yy in y..y+4{
            if img.get_pixel(xx,yy){
                code = code | bit;
            }
            bit = bit << 1;
        }
    }
    return code;
}

#[derive(Clone)]
struct BitImage{
    width: u32,
    height: u32,
    pixels: Vec<u8>
}




impl BitImage{
    fn new(w: u32, h:u32)->BitImage{
        BitImage{ width:w,
                  height:h,
                  pixels: vec![0;(w*h)as usize]}
    }
    fn get_pixel(&self, x: u32, y:u32)->u8{ self.pixels[(x+y*self.width) as usize] }
    fn set_pixel(& mut self, x: u32, y:u32, v: u8){
        self.pixels[(x+y*self.width) as usize] = v;
    }
    fn to_image(&self)->GrayImage{
        let mut img = GrayImage::new(self.width, self.height);
        for (x,y,pix) in img.enumerate_pixels_mut(){
            *pix = Luma([self.get_pixel(x,y)]);
        }
        return img;
    }
}



/*Calculate the likehood function of the bit map*/
fn evaluate_bitmap(bmp: &impl BinaryPixelAccess, hash: &TextureHash)->f64{
    let hash1 = readhash(bmp);
    let mut energy:f64=0.0;
    let inv1 = 1.0 / (hash1.total as f64);
    let inv  = 1.0 / (hash.total as f64);
    
    for i in 0..65536{
        let n = hash1.blockcounts[i] as f64;
        let m = hash.blockcounts[i] as f64;
        //calculate difference
        energy += (n*inv1 - m*inv).abs();
    }
    return energy;
}

fn randomize_bitmap(bmp: &mut BitImage, percent: f32){
    for i in 0..bmp.pixels.len(){
        if random::<f32>() < percent {
            bmp.pixels[i] ^= 255;
        }
    }
}

fn anneal(hash: &TextureHash, img: &mut BitImage, noise_percent: f32, anneal_steps: usize){

    let mut candidate:BitImage;// = img.clone();
    let mut energy = evaluate_bitmap(img, hash);
    
    for step in 0..anneal_steps{
        if step%100==0{
            println!("Step : {:?} of {:?} energy: {:?}",step, anneal_steps, energy);
        }
        //t varies from 0 to 1 uniformly
        let t = (step as f64)/((anneal_steps)as f64);
        candidate = img.clone();
        randomize_bitmap(&mut candidate, noise_percent);
        let cand_energy = evaluate_bitmap(&candidate, hash);
        //relative energy difference
        let denergy = cand_energy/energy-1.0;
        if (denergy < 0.0){
            *img = candidate.clone();
            energy = cand_energy;
        }
    }
}

fn main() {

    let img1:GrayImage = image::open("texture.png").unwrap().to_luma();

    /*
    // Construct a new RGB ImageBuffer with the specified width and height.
    let mut img: RgbImage = ImageBuffer::new(512, 512);
    // Put a pixel at coordinate (100, 100).
    let pixel = Rgb([255,0,0]);
    img.put_pixel(100, 100, pixel);
    // Write the contents of this image to the Writer in PNG format.
    //img.save("test.png").unwrap();
    println!("Hello, world!");
     */

    println!("dimensions {:?}", img1.dimensions());
    let hash = readhash(&img1);
    println!("hash {:?}", &hash.blockcounts[0..100]);
    println!("Total blocks: {:?}", hash.total);

    let mut bitimg = BitImage::new(128,128);
    println!("bit image dim: {:?}", bitimg.get_dimensions());
    println!("cross info: {:?}", evaluate_bitmap(&bitimg, &hash));
    println!("cross info with self: {:?}", evaluate_bitmap(&img1, &hash));

    //randomize_bitmap(&mut bitimg, 0.01);
    anneal(&hash, &mut bitimg, 0.0001, 100000);
    bitimg.to_image().save("out.png").unwrap();
    /*
    for cnt in hash.blockcounts.iter(){
        if *cnt != 0{
            println!("  {:?}", cnt)
        }
    }
     */
}
