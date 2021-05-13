# Experimental Rust firmware for Makerbase MKS TFT32_L V3.0

* [MKS Hardware](https://github.com/makerbase-mks/MKS-TFT-Hardware/tree/master/MKS%20TFT32/MKS%20TFT32_L%20V3.x)
* [LCD ILI9328](https://cdn-shop.adafruit.com/datasheets/ILI9328.pdf)

An experiment in reusing an LCD panel from an old 3D printer for other projects.
Only goes as far as initializing the display and providing a [DrawTarget](https://docs.rs/embedded-graphics/0.6.2/embedded_graphics/trait.DrawTarget.html) driver
to use with `embedded_graphics`.

TODO: fix HSE (board has 25Hhz but that input crashes `stm32f1xx_hal`).
