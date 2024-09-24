run:
	cargo build --bin esp32
	espflash flash --monitor target/xtensa-esp32-espidf/debug/esp32

deploy:
	cargo build
	espflash flash target/xtensa-esp32-espidf/debug/esp32

monitor:
	espflash monitor
