use anyhow::Error;
use db::{models::XmltvUrlRequest, Connection, CRUD, DB};
use quick_xml::{
    events::{attributes::Attributes, BytesStart, Event},
    name::QName,
    Reader, Writer,
};
use reqwest::Url;
use std::{io::Cursor, sync::Arc};

#[derive(Clone)]
pub struct XmlUtil {
    db: Arc<DB>,
}

impl XmlUtil {
    pub fn new(db: Arc<DB>) -> Self {
        XmlUtil { db }
    }

    pub async fn proxify_xmltv(&self, xml: &str, domain: String) -> Result<Vec<u8>, Error> {
        let mut reader = Reader::from_reader(xml.as_bytes());

        reader.trim_text(true);

        let mut writer = Writer::new(Cursor::new(Vec::new()));
        let mut buf = Vec::new();

        let mut tx = self.db.pool.begin().await?;

        loop {
            match reader.read_event_into_async(&mut buf).await {
                Ok(Event::Empty(e)) => {
                    if e.name().as_ref() == b"icon" {
                        let event = e.to_owned();

                        match event.try_get_attribute("src")? {
                            Some(src) => {
                                let value = src.unescape_value()?;

                                match Url::parse(&value) {
                                    Ok(url) => {
                                        if url.to_string().starts_with("http") {
                                            let icon_element = self
                                                .clone()
                                                .persist_proxify_xml_url(
                                                    e.attributes(),
                                                    url,
                                                    domain.clone(),
                                                    &mut tx,
                                                )
                                                .await?;

                                            writer.write_event(Event::Empty(icon_element))?;
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            None => (),
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Ok(e) => writer.write_event(e)?,
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            }
        }

        buf.clear();
        let bytes = writer.into_inner().into_inner();

        tx.commit().await?;

        Ok(bytes)
    }

    pub async fn persist_proxify_xml_url<'a>(
        self,
        attributes: Attributes<'_>,
        url: Url,
        domain: String,
        mut tx: &'a mut Connection,
    ) -> Result<BytesStart<'a>, Error> {
        let mut icon_element = BytesStart::new("icon");

        for attribute in attributes {
            let attr = attribute.unwrap();

            let id = self
                .db
                .xmltv_url
                .insert(
                    &mut tx,
                    XmltvUrlRequest {
                        url: url.to_string(),
                    },
                )
                .await?;

            if attr.key == QName(b"src") {
                icon_element
                    .push_attribute(("src", format!("http://{}/xmltv/{}", domain, id).as_str()));
            } else {
                icon_element.push_attribute(attr);
            }
        }

        Ok(icon_element)
    }
}
