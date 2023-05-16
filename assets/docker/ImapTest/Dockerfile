FROM ubuntu

RUN apt-get update &&\
	apt-get -y install\
	autoconf\
	automake\
	bison\
	curl\
	flex\
	gettext\
	git\
	libssl-dev\
	libtool\
	make\
	pkg-config\
	wget\
	zlib1g\
	zlib1g-dev

RUN git clone https://github.com/dovecot/core dovecot
WORKDIR dovecot
RUN ./autogen.sh
RUN PANDOC=false ./configure --enable-maintainer-mode 
RUN make
WORKDIR ..

RUN git clone https://github.com/dovecot/imaptest
WORKDIR imaptest
RUN ./autogen.sh
RUN ./configure --with-dovecot=../dovecot
RUN make 
RUN curl --location -O http://www.dovecot.org/tmp/dovecot-crlf
WORKDIR ..
