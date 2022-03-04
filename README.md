# Mandelbrot
Mandelbrot set image generator using multithreading or opencl

# Examples
Multithreading
##
```
$ ./target/release/mandelbrot --samples 4 --scale 0.02 -w 4096 -h 4096 --centrex -0.73 --centrey -0.2 --iterations 2048 -j 16 --name mandelbrot.jpg
Position (-0.73, -0.2) with scale 0.02 and 2048 iterations at size 4096x4096 16 samples per pixel 16 threads and colour code 7
Time taken: 34616ms
```

## Opencl
```
./target/release/mandelbrot --samples 4 --scale 0.02 -w 4096 -h 4096 --centrex -0.73 --centrey -0.2 --iterations 2048 --ocl --name mandelbrot.jpg
Position (-0.73, -0.2) with scale 0.02 and 2048 iterations at size 4096x4096 16 samples per pixel 1 threads and colour code 7
Running opencl version threads flag will be ignored and no progress bar can be shown
time taken: 1452ms
```
![mandelbrot](https://user-images.githubusercontent.com/12377096/156685264-4a390e71-2529-425c-bed1-1d96d22717f6.jpg)

