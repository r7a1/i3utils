FROM centos:8

USER root

RUN yum update -y \
    && yum install -y gcc curl \
    && rm -rf /var/cache/yum/* \
    && yum clean all

# See https://rust-lang.github.io/rustup-components-history/index.html to
# determine the best version.
ENV RUST_NIGHTLY_VERSION=2021-12-29
# https://github.com/mozilla/sccache/releases
ENV SCCACHE_VERSION=v0.2.15

ENV RUSTUP_HOME /opt/rust
ENV CARGO_HOME $RUSTUP_HOME
ENV PATH $PATH:$CARGO_HOME/bin
RUN bash -c "set -o pipefail && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile=minimal --default-toolchain nightly-$RUST_NIGHTLY_VERSION-x86_64-unknown-linux-gnu" \
    && echo export RUSTUP_HOME=$RUSTUP_HOME >> /etc/environment \
    && echo export CARGO_HOME=$CARGO_HOME >> /etc/environment \
    && echo export PATH=$PATH:$CARGO_HOME/bin >> /etc/profile

RUN yum check-update -y \
    && yum install -y pkgconfig openssl-devel \
    && rm -rf /var/cache/yum/* \
    && yum clean all

ENV SCCACHE_DIR $CARGO_HOME/sccache
RUN curl -sSfL https://github.com/mozilla/sccache/releases/download/$SCCACHE_VERSION/sccache-$SCCACHE_VERSION-x86_64-unknown-linux-musl.tar.gz \
    | tar xzvf - --strip-components 1 -C /opt/rust/bin/ sccache-$SCCACHE_VERSION-x86_64-unknown-linux-musl/sccache \
    && chmod +x /opt/rust/bin/sccache \
    && echo -e "[build]\nrustc-wrapper = \"$CARGO_HOME/bin/sccache\"" > $CARGO_HOME/config \
    && cat $CARGO_HOME/config

RUN yum check-update -y \
    && yum install -y cmake gcc-c++ libxcb-devel python3 freetype-devel expat-devel libxml2 rpmdevtools fontconfig-devel \
    && rm -rf /var/cache/yum/* \
    && yum clean all

RUN cargo install cargo-rpm \
    && rm -rf $CARGO_HOME/registry

WORKDIR "/workspaces"
RUN chmod 755 /workspaces

COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
