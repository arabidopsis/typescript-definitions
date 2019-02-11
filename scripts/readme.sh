#!/bin/bash
rustdoc --edition 2018 --crate-name typescript-definitions -o ./doc \
	--markdown-css ../normalize.css  \
	--markdown-css ../dark.css  \
	--html-before-content templates/before.html \
	--html-after-content templates/after.html \
	-L ./target/debug/ \
	-v README.md
