package spin

import (
	"fmt"
	"io"
	"os"
	"sync/atomic"
	"time"
)

const clearLine = "\r\033[K"
const resetColor = "\033[0m"

var defaultFrames = []rune{'|', '/', '-', '\\'}

type Spinner struct {
	frames []rune
	pos    int
	active uint64
	tpf    time.Duration
	writer io.Writer
	text   string
	prefix string
	suffix string
	color  string
	hidden uint64
}

type Option func(*Spinner)

func New(text string, opts ...Option) *Spinner {
	s := &Spinner{
		frames: defaultFrames,
		tpf:    100 * time.Millisecond,
		writer: os.Stdout,
		text:   text,
	}
	for _, o := range opts {
		o(s)
	}
	return s
}

func WithTimePerFrame(d time.Duration) Option {
	return func(s *Spinner) {
		s.tpf = d
	}
}
func WithWriter(w io.Writer) Option {
	return func(s *Spinner) {
		s.writer = w
	}
}
func WithFrames(frames []rune) Option {
	return func(s *Spinner) {
		s.frames = frames
	}
}

func (s *Spinner) Start() {
	if atomic.SwapUint64(&s.active, 1) == 1 {
		return
	}
	go func() {
		for atomic.LoadUint64(&s.active) == 1 {
			if atomic.LoadUint64(&s.hidden) == 1 {
				time.Sleep(s.tpf)
				continue
			}
			fmt.Fprintf(s.writer,
				"%s%s%s%c %s%s%s",
				clearLine,
				s.color,
				s.prefix,
				s.next(),
				s.text,
				s.suffix,
				resetColor)
			time.Sleep(s.tpf)
		}
	}()
}

func (s *Spinner) Stop() {
	atomic.StoreUint64(&s.active, 0)
	fmt.Fprint(s.writer, clearLine)
}

func (s *Spinner) StopWithMessage(msg string) {
	s.Stop()
	fmt.Fprintln(s.writer, msg)
}

func (s *Spinner) SetText(text string) {
	s.text = text
}

func (s *Spinner) SetPrefix(prefix string) {
	s.prefix = prefix
}

func (s *Spinner) SetSuffix(suffix string) {
	s.suffix = suffix
}

func (s *Spinner) SetColor(color string) {
	s.color = color
}

func (s *Spinner) Hide() {
	atomic.StoreUint64(&s.hidden, 1)
}

func (s *Spinner) Show() {
	atomic.StoreUint64(&s.hidden, 0)
}

func (s *Spinner) SetFrames(frames []rune) {
	s.frames = frames
	s.pos = 0
}

func (s *Spinner) AutoStop(d time.Duration) {
	go func() {
		time.Sleep(d)
		s.Stop()
	}()
}

func (s *Spinner) next() rune {
	r := s.frames[s.pos%len(s.frames)]
	s.pos++
	return r
}
