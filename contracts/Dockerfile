# Usage
# docker build . -t confio/cosmwasm-opt:latest
# docker run --rm -v PATH/TO/CONTRACT/Cargo.toml:/code confio/cosmwasm-opt:latest
# docker run --rm -v "$(pwd)":/code confio/cosmwasm-opt:latest
# docker run --rm -it -v "$(pwd)":/code confio/cosmwasm-opt:latest /bin/bash
FROM trzeci/emscripten:1.38.47

# setup rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
ENV PATH /root/.cargo/bin:$PATH
RUN rustup target add wasm32-unknown-unknown
RUN mkdir -p /root/.cargo/bin

# setup wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Assumes we mount the source code in /code
WORKDIR /code
ADD optimize.sh /root/optimize.sh
RUN chmod +x /root/optimize.sh

CMD ["/root/optimize.sh"]