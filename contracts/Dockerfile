FROM trzeci/emscripten:1.38.47

# setup rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
RUN rustup target add wasm32-unknown-unknown
RUN mkdir -p /root/.cargo/bin
ENV PATH /root/.cargo/bin:$PATH

# setup wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# install wasm-opt
RUN git clone https://github.com/webassembly/binaryen
RUN cd binaryen && cmake . && make
RUN cp binaryen/bin/wasm-opt /root/.cargo/bin/wasm-opt

# Assumes we mount the source code in /code
WORKDIR /code
ADD optimize.sh /root/optimize.sh
RUN chmod +x /root/optimize.sh

CMD ["/root/optimize.sh"]