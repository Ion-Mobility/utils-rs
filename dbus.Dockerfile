ARG CROSS_BASE_IMAGE
FROM $CROSS_BASE_IMAGE

ARG CROSS_DEB_ARCH

RUN dpkg --add-architecture $CROSS_DEB_ARCH
RUN apt-get update && apt-get -y install libdbus-1-dev:$CROSS_DEB_ARCH libssl-dev:$CROSS_DEB_ARCH libusb-1.0-0-dev:$CROSS_DEB_ARCH libftdi1-dev:$CROSS_DEB_ARCH libudev-dev:$CROSS_DEB_ARCH
