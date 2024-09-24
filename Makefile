deploy:
	cargo build
	espflash flash target/xtensa-esp32-espidf/debug/esp32

monitor:
	espflash monitor
