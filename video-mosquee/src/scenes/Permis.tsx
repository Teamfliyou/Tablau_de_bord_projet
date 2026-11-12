import React from 'react';
import {
	AbsoluteFill,
	Img,
	interpolate,
	spring,
	staticFile,
	useCurrentFrame,
	useVideoConfig,
} from 'remotion';
import {CheckMark, Kicker, MaskLine, Rise, SceneExit} from '../components/Common';
import {C, docFont, formatNumber, pop, progress, sans, serif} from '../theme';

export const PERMIS_DURATION = 390;

const LEFT = 112;

const titleStyle: React.CSSProperties = {
	fontFamily: serif,
	fontWeight: 700,
	fontSize: 100,
	lineHeight: 1.0,
	color: C.green,
	whiteSpace: 'nowrap',
};

const ARTICLE_TEXT = 'Le Permis de construire est accordé.';

const T = {
	doc: 30,
	header: 52,
	box: [80, 110] as const,
	type: [100, 150] as const,
	stamp: 160,
	stats: 120,
	caption: 180,
};

const StatCard: React.FC<{
	delay: number;
	width: number;
	value: React.ReactNode;
	label: string;
}> = ({delay, width, value, label}) => {
	const frame = useCurrentFrame();
	const {fps} = useVideoConfig();
	const s = spring({frame: frame - delay, fps, config: {damping: 15, stiffness: 120}});
	const bar = progress(frame, delay + 4, delay + 22);
	return (
		<div
			style={{
				position: 'relative',
				width,
				height: 116,
				background: C.white,
				borderRadius: 12,
				border: `2px solid ${C.border}`,
				padding: '14px 0 0 36px',
				overflow: 'hidden',
				opacity: Math.min(1, s * 1.5),
				transform: `translateY(${(1 - s) * 60}px)`,
			}}
		>
			<div
				style={{
					position: 'absolute',
					left: 0,
					bottom: 0,
					width: 9,
					height: `${bar * 100}%`,
					background: C.gold,
				}}
			/>
			<div style={{fontFamily: sans, fontWeight: 700, fontSize: 39, color: C.green, lineHeight: 1.3}}>
				{value}
			</div>
			<div style={{fontFamily: sans, fontWeight: 400, fontSize: 27, color: C.text}}>{label}</div>
		</div>
	);
};

const Document: React.FC = () => {
	const frame = useCurrentFrame();
	const {fps} = useVideoConfig();

	const enter = spring({frame: frame - T.doc, fps, config: {damping: 14, stiffness: 70}});
	const floatY = Math.sin(frame / 28) * 4;

	// petite secousse quand le tampon frappe
	const impact = frame - T.stamp - 5;
	const shake = impact >= 0 && impact < 12 ? Math.sin(impact * 2.4) * (12 - impact) * 0.5 : 0;

	const header = spring({frame: frame - T.header, fps, config: {damping: 200}});
	const box = progress(frame, ...T.box);
	const chars = Math.floor(
		interpolate(frame, T.type, [0, ARTICLE_TEXT.length], {
			extrapolateLeft: 'clamp',
			extrapolateRight: 'clamp',
		}),
	);
	const cursorOn = frame < T.type[1] + 10 && Math.floor(frame / 8) % 2 === 0;

	const stamp = spring({frame: frame - T.stamp, fps, config: {damping: 10, stiffness: 180}});
	const shine = progress(frame, T.stamp + 12, T.stamp + 40);

	return (
		<div
			style={{
				position: 'absolute',
				left: 912,
				top: 246,
				width: 896,
				transform: `translateX(${(1 - enter) * 700 + shake}px) translateY(${floatY}px) rotate(${interpolate(enter, [0, 1], [10, -1.2])}deg)`,
				opacity: Math.min(1, enter * 2),
			}}
		>
			<div
				style={{
					position: 'relative',
					height: 378,
					background: C.white,
					borderRadius: 6,
					boxShadow: '0 22px 60px rgba(15,61,46,0.16), 0 3px 10px rgba(0,0,0,0.06)',
					overflow: 'hidden',
				}}
			>
				{/* en-tête du document */}
				<div
					style={{
						position: 'absolute',
						left: 40,
						top: 40,
						right: 44,
						display: 'flex',
						alignItems: 'center',
						justifyContent: 'space-between',
						opacity: header,
						transform: `translateY(${(1 - header) * 16}px)`,
						fontFamily: docFont,
						fontWeight: 700,
						color: '#111',
					}}
				>
					<Img src={staticFile('blason.png')} style={{width: 140}} />
					<div style={{fontSize: 23, textAlign: 'center', lineHeight: 2.2}}>
						COMMUNE DE
						<br />
						LONGUENESSE
					</div>
					<div style={{textAlign: 'right'}}>
						<div style={{fontSize: 25, lineHeight: 1.2}}>
							DELIVRANCE D&apos;UN PERMIS DE
							<br />
							CONSTRUIRE
						</div>
						<div style={{fontSize: 19, marginTop: 12}}>PAR LE MAIRE AU NOM DE LA COMMUNE</div>
					</div>
				</div>
				{/* points de suspension */}
				<div
					style={{
						position: 'absolute',
						top: 176,
						left: 0,
						right: 0,
						display: 'flex',
						justifyContent: 'center',
						gap: 52,
					}}
				>
					{[0, 1, 2].map((d) => (
						<div
							key={d}
							style={{
								width: 10,
								height: 10,
								borderRadius: '50%',
								background: '#d8d8d8',
								opacity: progress(frame, T.header + 10 + d * 5, T.header + 18 + d * 5),
							}}
						/>
					))}
				</div>

				{/* cadre "Article 1" qui se dessine */}
				<svg
					width={824}
					height={124}
					style={{position: 'absolute', left: 36, top: 216, overflow: 'visible'}}
				>
					<rect
						x={2}
						y={2}
						width={820}
						height={118}
						rx={14}
						fill="none"
						stroke={C.gold}
						strokeWidth={5}
						pathLength={1}
						strokeDasharray={1}
						strokeDashoffset={1 - box}
					/>
				</svg>
				<div
					style={{
						position: 'absolute',
						left: 72,
						top: 252,
						fontFamily: docFont,
						fontSize: 35,
						color: '#1a1a1a',
						whiteSpace: 'nowrap',
						opacity: progress(frame, T.box[0] + 10, T.box[0] + 20),
					}}
				>
					<span style={{fontWeight: 700, textDecoration: 'underline'}}>Article 1</span>
					<span> : {ARTICLE_TEXT.slice(0, chars)}</span>
					<span style={{opacity: cursorOn ? 1 : 0, color: C.gold}}>|</span>
				</div>

				{/* reflet lumineux après validation */}
				<div
					style={{
						position: 'absolute',
						top: -100,
						bottom: -100,
						width: 160,
						left: interpolate(shine, [0, 1], [-250, 1100]),
						background:
							'linear-gradient(90deg, rgba(255,255,255,0) 0%, rgba(255,240,210,0.65) 50%, rgba(255,255,255,0) 100%)',
						transform: 'rotate(18deg)',
						opacity: shine > 0 && shine < 1 ? 1 : 0,
					}}
				/>
			</div>

			{/* Tampon : coche verte */}
			<div
				style={{
					position: 'absolute',
					right: -52,
					top: 282,
					width: 128,
					height: 128,
					borderRadius: '50%',
					background: C.green,
					border: `7px solid ${C.gold}`,
					boxShadow: '0 12px 30px rgba(0,0,0,0.25)',
					display: 'flex',
					alignItems: 'center',
					justifyContent: 'center',
					opacity: Math.min(1, stamp * 3),
					transform: `scale(${interpolate(stamp, [0, 1], [2.6, 1])}) rotate(${interpolate(stamp, [0, 1], [-30, 0])}deg)`,
				}}
			>
				<CheckMark size={104} stroke={11} progress={progress(frame, T.stamp + 4, T.stamp + 16)} />
			</div>
			{/* onde du tampon */}
			<div
				style={{
					position: 'absolute',
					right: -52,
					top: 282,
					width: 128,
					height: 128,
					borderRadius: '50%',
					border: `5px solid ${C.gold}`,
					opacity: frame > T.stamp + 5 ? interpolate(frame, [T.stamp + 5, T.stamp + 30], [0.8, 0], {extrapolateRight: 'clamp'}) : 0,
					transform: `scale(${interpolate(frame, [T.stamp + 5, T.stamp + 30], [1, 2.2], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'})})`,
				}}
			/>
		</div>
	);
};

export const Permis: React.FC = () => {
	const frame = useCurrentFrame();
	const {fps} = useVideoConfig();

	const highlight = progress(frame, 42, 66);
	const m2 = interpolate(frame, [T.stats + 18, T.stats + 18 + 50], [0, 378], {
		extrapolateLeft: 'clamp',
		extrapolateRight: 'clamp',
		easing: (t) => 1 - Math.pow(1 - t, 3),
	});
	const checkPop = pop(frame, fps, T.stamp);

	return (
		<SceneExit duration={PERMIS_DURATION}>
			<AbsoluteFill>
				<div style={{position: 'absolute', left: LEFT, top: 132}}>
					<Kicker text="BONNE NOUVELLE" ar="الحمد لله" />
				</div>

				<div style={{position: 'absolute', left: LEFT - 2, top: 214}}>
					<MaskLine delay={12}>
						<div style={titleStyle}>Le permis de</div>
					</MaskLine>
					<MaskLine delay={19} style={{marginTop: -6}}>
						<div style={titleStyle}>construire est</div>
					</MaskLine>
					<MaskLine delay={26} style={{marginTop: -6}}>
						<div style={{...titleStyle, position: 'relative', display: 'inline-block'}}>
							<div
								style={{
									position: 'absolute',
									left: 0,
									bottom: 6,
									height: 26,
									width: `${highlight * 100}%`,
									background: C.goldSoft,
									zIndex: 0,
								}}
							/>
							<span
								style={{
									position: 'relative',
									zIndex: 1,
									display: 'inline-block',
									transform: `scale(${1 + (checkPop > 0.01 ? Math.sin(Math.min(1, checkPop) * Math.PI) * 0.04 : 0)})`,
									transformOrigin: 'left bottom',
								}}
							>
								accordé
							</span>
						</div>
					</MaskLine>
				</div>

				<div style={{position: 'absolute', left: LEFT, top: 540, width: 740}}>
					<Rise delay={48} distance={30}>
						<div
							style={{
								fontFamily: sans,
								fontWeight: 400,
								fontSize: 40,
								lineHeight: 1.33,
								color: C.text,
							}}
						>
							Aménagement de notre lieu de culte, avec un nouvel étage et une extension.
						</div>
					</Rise>
				</div>

				<div style={{position: 'absolute', left: LEFT, top: 718, display: 'flex', gap: 22}}>
					<StatCard delay={T.stats} width={400} value="4 novembre 2026" label="Arrêté du maire" />
					<StatCard
						delay={T.stats + 12}
						width={300}
						value={<span style={{fontVariantNumeric: 'tabular-nums'}}>+ {formatNumber(m2)} m²</span>}
						label="de surface créée"
					/>
				</div>

				<Document />

				<div style={{position: 'absolute', left: 914, top: 680}}>
					<Rise delay={T.caption} distance={16}>
						<div style={{fontFamily: sans, fontWeight: 400, fontSize: 31, color: C.text}}>
							Extrait de l&apos;arrêté officiel — Mairie de Longuenesse
						</div>
					</Rise>
				</div>
			</AbsoluteFill>
		</SceneExit>
	);
};
