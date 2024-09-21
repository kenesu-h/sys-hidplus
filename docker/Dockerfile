FROM devkitpro/devkita64:20240916

ARG BASE_PATH="atmosphere/contents/4200000000001312"

WORKDIR /app
COPY . .

RUN mkdir -p /tmp/$BASE_PATH/flags
RUN make
RUN mv app.nsp /tmp/$BASE_PATH/exefs.nsp
RUN touch /tmp/$BASE_PATH/flags/boot2.flag

WORKDIR /tmp
RUN zip -r release_$(date +"%Y%m%d_%H%M%S").zip ./*

WORKDIR /app
CMD ["bash", "-c", "mv /tmp/release_*.zip release/"]
