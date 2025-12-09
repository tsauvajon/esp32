generate:
	esp-generate --chip esp32 -o embassy -o esp-backtrace -o ci -o vscode -o unstable-hal -o log

generate-c3:
	esp-generate --chip esp32c3 -o embassy -o esp-backtrace -o ci -o vscode -o unstable-hal -o log
