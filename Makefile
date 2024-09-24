deploy:
	cargo build
	espflash flash target/xtensa-esp32-espidf/debug/test2

monitor:
	espflash monitor
