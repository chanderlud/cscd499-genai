use windows::core::{Result, HSTRING};
use windows::Graphics::Imaging::BitmapDecoder;
use windows::Media::Ocr::OcrEngine;
use windows::Storage::{FileAccessMode, StorageFile};

#[derive(Debug, Clone)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct WordRect {
    pub text: String,
    pub rect: Rect,
}

pub async fn ocr_with_bounding_boxes(path: &str) -> Result<Vec<WordRect>> {
    let path_hstring = HSTRING::from(path);

    // Open the file
    let file = StorageFile::GetFileFromPathAsync(&path_hstring)?.await?;

    // Open stream for reading
    let stream = file.OpenAsync(FileAccessMode::Read)?.await?;

    // Create bitmap decoder
    let decoder = BitmapDecoder::CreateAsync(&stream)?.await?;

    // Get software bitmap
    let bitmap = decoder.GetSoftwareBitmapAsync()?.await?;

    // Create OCR engine with default language
    let ocr_engine = OcrEngine::TryCreateFromUserProfileLanguages()?;

    // Perform OCR - pass bitmap by reference
    let result = ocr_engine.RecognizeAsync(&bitmap)?.await?;

    // Extract words with bounding boxes
    let mut words = Vec::new();
    let lines = result.Lines()?;

    for i in 0..lines.Size()? {
        let line = lines.GetAt(i)?;
        let word_bounds = line.Words()?;

        for j in 0..word_bounds.Size()? {
            let word = word_bounds.GetAt(j)?;
            let text = word.Text()?.to_string();
            let rect = word.BoundingRect()?;

            words.push(WordRect {
                text,
                rect: Rect {
                    x: rect.X,
                    y: rect.Y,
                    width: rect.Width,
                    height: rect.Height,
                },
            });
        }
    }

    Ok(words)
}
