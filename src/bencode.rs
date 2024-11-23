use bendy::{
    decoding::{Error as BencodeError, FromBencode, Object},
    encoding::AsString,
};

#[derive(Debug, PartialEq)]
pub struct Bencode {
    announce: String,
    info: Info,
}

// TODO: create custom errors
impl Bencode {
    pub fn parse(input: &[u8]) -> Self {
        Self::from_bencode(input).unwrap_or_else(|err| {
            panic!("Error parsing bencode: {:?}", err);
        })
    }
}

#[derive(Debug, PartialEq)]
struct File {
    length: u64,
    path: Vec<String>,
}

#[derive(Debug, PartialEq)]
enum Files {
    Single(u64),
    Multiple(Vec<File>),
}

#[derive(Debug, PartialEq)]
struct Info {
    name: String,
    piece_length: u64,
    files: Files,
    pieces: Vec<u8>,
}

impl FromBencode for Bencode {
    fn decode_bencode_object(object: Object) -> Result<Self, BencodeError>
    where
        Self: Sized,
    {
        let mut announce = None;
        let mut info = None;

        let mut dict_dec = object.try_into_dictionary()?;
        while let Some(pair) = dict_dec.next_pair()? {
            match pair {
                (b"announce", value) => {
                    announce = String::decode_bencode_object(value).map(Some)?;
                }
                (b"info", value) => {
                    info = Info::decode_bencode_object(value).map(Some)?;
                }
                (_, _) => {}
            }
        }

        let announce = announce.ok_or_else(|| BencodeError::missing_field("announce"))?;
        let info = info.ok_or_else(|| BencodeError::missing_field("info"))?;

        Ok(Bencode { announce, info })
    }
}

impl FromBencode for Info {
    const EXPECTED_RECURSION_DEPTH: usize = 1;

    fn decode_bencode_object(object: Object) -> Result<Self, BencodeError>
    where
        Self: Sized,
    {
        let mut length = None;
        let mut name = None;
        let mut piece_length = None;
        let mut pieces = None;
        let mut files = Vec::new();

        let mut dict_dec = object.try_into_dictionary()?;
        while let Some(pair) = dict_dec.next_pair()? {
            match pair {
                (b"length", value) => {
                    length = value
                        .try_into_integer()
                        // TODO: handle error
                        .map(|value| value.parse::<u64>().unwrap())
                        .map(Some)?;
                }
                (b"name", value) => {
                    name = String::decode_bencode_object(value).map(Some)?;
                }
                (b"piece length", value) => {
                    piece_length = value
                        .try_into_integer()
                        // TODO: handle error
                        .map(|value| value.parse::<u64>().unwrap())
                        .map(Some)?;
                }
                (b"pieces", value) => {
                    pieces = AsString::decode_bencode_object(value).map(|bytes| Some(bytes.0))?;
                }
                (b"files", value) => {
                    let mut f = value.try_into_list()?;
                    while let Some(item) = f.next_object()? {
                        let file = File::decode_bencode_object(item)?;

                        files.push(file);
                    }
                }
                (_, _) => {}
            }
        }

        let name = name.ok_or_else(|| BencodeError::missing_field("name"))?;
        let piece_length =
            piece_length.ok_or_else(|| BencodeError::missing_field("piece_length"))?;
        let pieces = pieces.ok_or_else(|| BencodeError::missing_field("pieces"))?;
        let files = if files.is_empty() {
            // TODO: handle error
            Files::Single(length.unwrap())
        } else {
            Files::Multiple(files)
        };

        Ok(Info {
            name,
            piece_length,
            files,
            pieces,
        })
    }
}

impl FromBencode for File {
    fn decode_bencode_object(object: Object) -> Result<Self, BencodeError>
    where
        Self: Sized,
    {
        let mut length = None;
        let mut path = None;

        let mut dict_dec = object.try_into_dictionary()?;
        while let Some(pair) = dict_dec.next_pair()? {
            match pair {
                (b"length", value) => {
                    length = value
                        .try_into_integer()
                        // TODO: handle error
                        .map(|value| value.parse::<u64>().unwrap())
                        .map(Some)?
                }
                (b"path", value) => {
                    path = Vec::decode_bencode_object(value).map(Some)?;
                }
                (_, _) => {}
            }
        }

        let length = length.ok_or_else(|| BencodeError::missing_field("length"))?;
        let path = path.ok_or_else(|| BencodeError::missing_field("path"))?;

        Ok(File { length, path })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_penguin_torrent() {
        let file_content = fs::read("torrent_files/penguin.torrent").unwrap_or_else(|err| {
            panic!("Error reading file: {:?}", err);
        });
        let parsed_bencode = Bencode::parse(&file_content);
        let expected_info = Info {
            name: "The.Penguin.S01.WEBDL.720p".to_string(),
            piece_length: 8388608,
            files: Files::Multiple(vec![
                File {
                    length: 3698684676,
                    path: vec!["The.Penguin.S01E01.WEBDL.720p.RGzsRutracker.mkv".to_string()],
                },
                File {
                    length: 3187627216,
                    path: vec!["The.Penguin.S01E02.WEBDL.720p.RGzsRutracker.mkv".to_string()],
                },
                File {
                    length: 3327956522,
                    path: vec!["The.Penguin.S01E03.WEBDL.720p.RGzsRutracker.mkv".to_string()],
                },
                File {
                    length: 3229360143,
                    path: vec!["The.Penguin.S01E04.WEBDL.720p.RGzsRutracker.mkv".to_string()],
                },
                File {
                    length: 2984092968,
                    path: vec!["The.Penguin.S01E05.WEBDL.720p.RGzsRutracker.mkv".to_string()],
                },
                File {
                    length: 2739121133,
                    path: vec!["The.Penguin.S01E06.WEBDL.720p.RGzsRutracker.mkv".to_string()],
                },
                File {
                    length: 2619972834,
                    path: vec!["The.Penguin.S01E07.WEBDL.720p.RGzsRutracker.mkv".to_string()],
                },
                File {
                    length: 3518076714,
                    path: vec!["The.Penguin.S01E08.WEBDL.720p.RGzsRutracker.mkv".to_string()],
                },
            ]),
            pieces: vec![],
        };

        assert_eq!(parsed_bencode.announce, "http://bt2.t-ru.org/ann");
        assert_eq!(parsed_bencode.info.name, expected_info.name);
        assert_eq!(parsed_bencode.info.piece_length, expected_info.piece_length);
        assert_eq!(parsed_bencode.info.files, expected_info.files);
        assert_eq!(parsed_bencode.info.pieces.len(), 60340);
    }

    #[test]
    fn test_inception_torrent() {
        let file_content = fs::read("torrent_files/inception.torrent").unwrap_or_else(|err| {
            panic!("Error reading file: {:?}", err);
        });

        let parsed_bencode = Bencode::parse(&file_content);
        let expected_info = Info {
            name: "Inception.2010.2160p.UHD.BDRip.HDR.x265.DD+5.1-VoX.mkv".to_string(),
            piece_length: 8388608,
            files: Files::Single(40580383319),
            pieces: vec![],
        };

        assert_eq!(parsed_bencode.announce, "http://bt2.t-ru.org/ann");
        assert_eq!(parsed_bencode.info.files, expected_info.files);
        assert_eq!(parsed_bencode.info.name, expected_info.name);
        assert_eq!(parsed_bencode.info.piece_length, expected_info.piece_length);
        assert_eq!(parsed_bencode.info.files, expected_info.files);
        assert_eq!(parsed_bencode.info.pieces.len(), 96760);
    }

    #[test]
    fn test_sample_torrent() {
        let file_content = fs::read("torrent_files/sample.torrent").unwrap_or_else(|err| {
            panic!("Error reading file: {:?}", err);
        });

        let parsed_bencode = Bencode::parse(&file_content);
        let expected_info = Info {
            name: "sample.txt".to_string(),
            piece_length: 32768,
            files: Files::Single(92063),
            pieces: vec![],
        };

        assert_eq!(
            parsed_bencode.announce,
            "http://bittorrent-test-tracker.codecrafters.io/announce"
        );
        assert_eq!(parsed_bencode.info.files, expected_info.files);
        assert_eq!(parsed_bencode.info.name, expected_info.name);
        assert_eq!(parsed_bencode.info.piece_length, expected_info.piece_length);
        assert_eq!(parsed_bencode.info.files, expected_info.files);
        assert_eq!(parsed_bencode.info.pieces.len(), 60);
    }
}
