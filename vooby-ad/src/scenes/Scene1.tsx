import React from 'react';
import {Img, staticFile} from 'remotion';
import {Photo} from '../components/Photo';
import {GoldRule, Label, Reveal, Title} from '../components/Text';
import {SceneShell, Shade, Vignette} from '../components/Scene';
import {BEAT} from '../constants';

export const Scene1: React.FC<{duration: number}> = ({duration}) => (
	<SceneShell fadeIn={false}>
		<Photo src="photos/04-ambiance-droite.jpg" from={{scale: 1.04, x: 0, y: 0}} to={{scale: 1.26, x: -2, y: -1.5}} duration={duration} origin="45% 60%" />
		<Shade top={0.55} bottom={0.5} flat={0.12} />
		<Vignette />
		<div style={{position: 'absolute', left: 0, right: 0, top: 430, display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 28}}>
			<Reveal at={Math.round(BEAT * 1) + 2}>
				<Label size={34}>Édition 2026</Label>
			</Reveal>
			<Reveal at={Math.round(BEAT * 1.5)}>
				<Title size={150} style={{letterSpacing: '0.06em', lineHeight: 0.95}}>BLACK</Title>
			</Reveal>
			<Reveal at={Math.round(BEAT * 2.5)} style={{marginTop: -30}}>
				<Title size={150} style={{letterSpacing: '0.04em', lineHeight: 0.95}}>NOVEMBER</Title>
			</Reveal>
			<GoldRule at={Math.round(BEAT * 4)} width={180} />
			<Reveal at={Math.round(BEAT * 5)}>
				<Img src={staticFile('logo/vooby-logo-blanc.png')} style={{width: 430, filter: 'drop-shadow(0 2px 14px rgba(0,0,0,0.5))'}} />
			</Reveal>
		</div>
	</SceneShell>
);
