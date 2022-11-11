$pdf_mode=1;
$out_dir='out';

$pdflatex = 'pdflatex -interaction nonstopmode %O %S';
ensure_path('TEXINPUTS');