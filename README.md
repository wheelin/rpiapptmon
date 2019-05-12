# rpiapptmon

Raspberry Pi appartement monitor. Monitors several phyisical variables and makes them available through a webpage with graphs. A sense hat is used for its matrix display, the IMU unit and the humidity sensor.  

## Air quality
MICS-5524 https://cdn-shop.adafruit.com/product-files/3199/MiCS-5524.pdf
ADS1115 http://www.ti.com/lit/ds/symlink/ads1115.pdf I2C address : 0x48

## Light (color)
TCS34725 https://cdn-shop.adafruit.com/datasheets/TCS34725.pdf. I2C address : 0x29

## Pressure and temperature (off-board)
BMP180 https://ae-bst.resource.bosch.com/media/_tech/media/datasheets/BST-BMP180-DS000.pdf. I2C address : 0x77 
This sensor is added to the system because the sense hat temperature sensors are heaten up by the raspberry pi processor. 

## Pressure and temperature (on-board)
LPS25H https://www.mouser.ch/ds/2/389/lps25h-955105.pdf. I2C address : 0x5C. 

## Inertial module 
LSM9DS1 https://cdn.sparkfun.com/assets/learn_tutorials/3/7/3/LSM9DS1_Datasheet.pdf. I2C address : 0x1C 

## Humidity 
HTS221 http://www.farnell.com/datasheets/2046114.pdf. I2C address : 0x5F

## LED matrix
I2C address : 0x46. Can also be acceded through a character device frame buffer in /dev/fb1.

